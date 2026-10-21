// @ts-check

/**
 * @param {string[]} argv
 * @returns {string}
 */
function run(argv) {
  try {
    const request = parseRequest(argv);
    const handler = HANDLERS[request.action];
    if (!handler) {
      fail("unsupported_action", "The requested action is not supported");
    }

    const mail = Application("Mail");
    return JSON.stringify({ ok: true, data: handler(mail, request) });
  } catch (error) {
    return JSON.stringify(errorEnvelope(error));
  }
}

/** @type {Readonly<Record<string, BridgeHandler>>} */
const HANDLERS = {
  accounts: listAccounts,
  mailboxes: listMailboxes,
  unread: listUnread,
  search: searchMessages,
  get: getMessage,
  "mark-read": markRead,
  move: moveMessage,
};

/**
 * @param {string[]} argv
 * @returns {BridgeRequest}
 */
function parseRequest(argv) {
  if (!Array.isArray(argv) || argv.length !== 1) {
    fail("invalid_request", "Exactly one JSON request argument is required");
  }
  const argument = argv[0];
  if (argument === undefined) {
    fail("invalid_request", "The request must be valid JSON");
  }

  /** @type {unknown} */
  let request;
  try {
    request = JSON.parse(argument);
  } catch {
    fail("invalid_request", "The request must be valid JSON");
  }
  if (!request || Array.isArray(request) || typeof request !== "object") {
    fail("invalid_request", "The request must be a JSON object");
  }
  const record = /** @type {Record<string, unknown>} */ (request);
  requireText(record.action, "action", 64);
  return /** @type {BridgeRequest} */ (/** @type {unknown} */ (record));
}

/**
 * @param {MailApplication} mail
 * @returns {Array<Record<string, unknown>>}
 */
function listAccounts(mail) {
  const accounts = mail.accounts();
  if (accounts.length > 256) {
    fail("account_count_exceeded", "Account count exceeds the supported limit");
  }
  return accounts.map(function (account) {
    return {
      id: boundedString(account.id(), 1_024),
      name: boundedString(account.name(), 1_024),
      email_addresses: account
        .emailAddresses()
        .slice(0, 256)
        .map(function (address) {
          return boundedString(address, 1_024);
        }),
      enabled: Boolean(account.enabled()),
      type: boundedString(account.accountType(), 256),
    };
  });
}

/**
 * @param {MailApplication} mail
 * @param {BridgeRequest} request
 * @returns {MailboxSummary[]}
 */
function listMailboxes(mail, request) {
  const account = resolveAccount(mail, requireText(request.account, "account", 1_024));
  const result = /** @type {MailboxSummary[]} */ ([]);
  appendMailboxes(account.mailboxes(), [], result);
  result.sort(function (left, right) {
    return left.path.localeCompare(right.path);
  });
  return result;
}

/**
 * @param {MailMailbox[]} mailboxes
 * @param {string[]} parents
 * @param {MailboxSummary[]} result
 * @returns {void}
 */
function appendMailboxes(mailboxes, parents, result) {
  if (parents.length > 32) {
    fail("mailbox_depth_exceeded", "Mailbox nesting exceeds the supported depth");
  }
  for (const mailbox of mailboxes) {
    if (result.length >= 2_000) {
      fail("mailbox_count_exceeded", "Mailbox count exceeds the supported limit");
    }
    const name = String(mailbox.name());
    const segments = parents.concat([name]);
    result.push({
      path: encodeMailboxPath(segments),
      name: name,
      unread_count: Number(mailbox.unreadCount()),
    });
    appendMailboxes(mailbox.mailboxes(), segments, result);
  }
}

/**
 * @param {MailApplication} mail
 * @param {BridgeRequest} request
 * @returns {MessageSummary[]}
 */
function listUnread(mail, request) {
  const limit = requireInteger(request.limit, "limit", { minimum: 1, maximum: 200 });
  const accountId = optionalText(request.account, "account", 1_024);
  const messages = mail.inbox().messages.whose({ readStatus: false })();
  return summarizeAndLimit(messages, accountId, limit);
}

/**
 * @param {MailApplication} mail
 * @param {BridgeRequest} request
 * @returns {MessageSummary[]}
 */
function searchMessages(mail, request) {
  const query = requireText(request.query, "query", 1_024);
  const limit = requireInteger(request.limit, "limit", { minimum: 1, maximum: 200 });
  const accountId = optionalText(request.account, "account", 1_024);
  const requestedMailboxPath = optionalText(request.mailbox, "mailbox", 4_096);
  if (requestedMailboxPath && !accountId) {
    fail("invalid_request", "account is required when mailbox is specified");
  }

  const source = requestedMailboxPath
    ? resolveMailbox(resolveAccount(mail, /** @type {string} */ (accountId)), requestedMailboxPath)
    : mail.inbox();
  const messages = source.messages.whose({
    _or: [{ subject: { _contains: query } }, { sender: { _contains: query } }],
  })();
  return summarizeAndLimit(messages, accountId, limit);
}

/**
 * @param {MailApplication} mail
 * @param {BridgeRequest} request
 * @returns {MessageSummary}
 */
function getMessage(mail, request) {
  const locator = requireLocator(request);
  const includeBody = requireBoolean(request.include_body, "include_body");
  const message = resolveMessage(mail, locator);
  const summary = summarizeMessage(message);
  if (!includeBody) {
    return summary;
  }

  const maxBodyBytes = requireInteger(request.max_body_bytes, "max_body_bytes", {
    minimum: 1,
    maximum: 1_000_000,
  });
  const body = truncateUtf8(String(message.content()), maxBodyBytes);
  summary.body = body.text;
  summary.body_truncated = body.isTruncated;
  return summary;
}

/**
 * @param {MailApplication} mail
 * @param {BridgeRequest} request
 * @returns {MessageSummary}
 */
function markRead(mail, request) {
  const message = resolveMessage(mail, requireLocator(request));
  /** @type {{readStatus: boolean}} */ (/** @type {unknown} */ (message)).readStatus = true;
  if (!message.readStatus()) {
    fail("mutation_unverified", "Mail did not confirm the read status change");
  }
  return summarizeMessage(message);
}

/**
 * @param {MailApplication} mail
 * @param {BridgeRequest} request
 * @returns {MessageSummary}
 */
function moveMessage(mail, request) {
  const locator = requireLocator(request);
  const destinationPath = requireText(request.destination, "destination", 4_096);
  if (destinationPath === locator.mailbox) {
    fail("invalid_request", "Source and destination mailboxes must differ");
  }

  const account = resolveAccount(mail, locator.account);
  const message = resolveMessageInAccount(account, locator);
  const messageId = messageIdForVerification(message);
  const destination = resolveMailbox(account, destinationPath);
  const moved = mail.move(message, { to: destination });
  const verified =
    findMessage(destination, locator.id) || findMessageByMessageId(destination, messageId);
  if (verified) {
    return summarizeMessage(verified);
  }
  try {
    if (moved && mailboxPath(moved.mailbox()) === destinationPath) {
      return summarizeMessage(moved);
    }
  } catch {
    // Some providers invalidate the original object specifier after a move.
  }
  fail(
    "move_unverified",
    "Mail accepted the move but its result could not be verified; inspect Mail before retrying",
  );
}

/**
 * @param {MailMessage[]} messages
 * @param {string | null} accountId
 * @param {number} limit
 * @returns {MessageSummary[]}
 */
function summarizeAndLimit(messages, accountId, limit) {
  const summaries = [];
  for (const message of messages) {
    if (accountId && String(message.mailbox().account().id()) !== accountId) {
      continue;
    }
    summaries.push(summarizeMessage(message));
    if (summaries.length > limit) {
      summaries.sort(compareNewestFirst);
      summaries.pop();
    }
  }
  summaries.sort(compareNewestFirst);
  return summaries;
}

/**
 * @param {MessageSummary} left
 * @param {MessageSummary} right
 * @returns {number}
 */
function compareNewestFirst(left, right) {
  return right.date_received.localeCompare(left.date_received);
}

/**
 * @param {MailMessage} message
 * @returns {MessageSummary}
 */
function summarizeMessage(message) {
  const mailbox = message.mailbox();
  return {
    account: boundedString(mailbox.account().id(), 1_024),
    mailbox: mailboxPath(mailbox),
    id: Number(message.id()),
    message_id: boundedString(message.messageId(), 2_048),
    sender: boundedString(message.sender(), 2_048),
    subject: boundedString(message.subject(), 4_096),
    date_received: isoDate(message.dateReceived()),
    read: Boolean(message.readStatus()),
    size_bytes: Number(message.messageSize()),
  };
}
