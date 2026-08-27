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

const HANDLERS = {
  accounts: listAccounts,
  mailboxes: listMailboxes,
  unread: listUnread,
  search: searchMessages,
  get: getMessage,
  "mark-read": markRead,
  move: moveMessage,
};

function parseRequest(argv) {
  if (!Array.isArray(argv) || argv.length !== 1) {
    fail("invalid_request", "Exactly one JSON request argument is required");
  }

  let request;
  try {
    request = JSON.parse(argv[0]);
  } catch (_) {
    fail("invalid_request", "The request must be valid JSON");
  }
  if (!request || Array.isArray(request) || typeof request !== "object") {
    fail("invalid_request", "The request must be a JSON object");
  }
  requireText(request.action, "action", 64);
  return request;
}

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

function listMailboxes(mail, request) {
  const account = resolveAccount(mail, requireText(request.account, "account", 1_024));
  const result = [];
  appendMailboxes(account.mailboxes(), [], result, 0);
  result.sort(function (left, right) {
    return left.path.localeCompare(right.path);
  });
  return result;
}

function appendMailboxes(mailboxes, parents, result, depth) {
  if (depth > 32) {
    fail("mailbox_depth_exceeded", "Mailbox nesting exceeds the supported depth");
  }
  for (let index = 0; index < mailboxes.length; index += 1) {
    if (result.length >= 2_000) {
      fail("mailbox_count_exceeded", "Mailbox count exceeds the supported limit");
    }
    const mailbox = mailboxes[index];
    const name = String(mailbox.name());
    const segments = parents.concat([name]);
    result.push({
      path: encodeMailboxPath(segments),
      name: name,
      unread_count: Number(mailbox.unreadCount()),
    });
    appendMailboxes(mailbox.mailboxes(), segments, result, depth + 1);
  }
}

function listUnread(mail, request) {
  const limit = requireInteger(request.limit, "limit", 1, 200);
  const accountId = optionalText(request.account, "account", 1_024);
  const messages = mail.inbox().messages.whose({ readStatus: false })();
  return summarizeAndLimit(messages, accountId, limit);
}

function searchMessages(mail, request) {
  const query = requireText(request.query, "query", 1_024);
  const limit = requireInteger(request.limit, "limit", 1, 200);
  const accountId = optionalText(request.account, "account", 1_024);
  const mailboxPath = optionalText(request.mailbox, "mailbox", 4_096);
  if (mailboxPath && !accountId) {
    fail("invalid_request", "account is required when mailbox is specified");
  }

  const source = mailboxPath
    ? resolveMailbox(resolveAccount(mail, accountId), mailboxPath)
    : mail.inbox();
  const messages = source.messages.whose({
    _or: [{ subject: { _contains: query } }, { sender: { _contains: query } }],
  })();
  return summarizeAndLimit(messages, accountId, limit);
}

function getMessage(mail, request) {
  const locator = requireLocator(request);
  const maxBodyBytes = requireInteger(request.max_body_bytes, "max_body_bytes", 1, 1_000_000);
  const message = resolveMessage(mail, locator);
  const summary = summarizeMessage(message);
  const body = truncateUtf8(String(message.content()), maxBodyBytes);
  summary.body = body.text;
  summary.body_truncated = body.isTruncated;
  return summary;
}

function markRead(mail, request) {
  const message = resolveMessage(mail, requireLocator(request));
  message.readStatus = true;
  if (!message.readStatus()) {
    fail("mutation_unverified", "Mail did not confirm the read status change");
  }
  return summarizeMessage(message);
}

function moveMessage(mail, request) {
  const locator = requireLocator(request);
  const destinationPath = requireText(request.destination, "destination", 4_096);
  if (destinationPath === locator.mailbox) {
    fail("invalid_request", "Source and destination mailboxes must differ");
  }

  const account = resolveAccount(mail, locator.account);
  const message = resolveMessageInAccount(account, locator);
  const destination = resolveMailbox(account, destinationPath);
  const moved = mail.move(message, { to: destination });
  const verified = findMessage(destination, locator.id);
  if (verified) {
    return summarizeMessage(verified);
  }
  try {
    if (moved && mailboxPath(moved.mailbox()) === destinationPath) {
      return summarizeMessage(moved);
    }
  } catch (_) {
    // Some providers invalidate the original object specifier after a move.
  }
  fail(
    "move_unverified",
    "Mail accepted the move but its result could not be verified; inspect Mail before retrying",
  );
}

function summarizeAndLimit(messages, accountId, limit) {
  const summaries = [];
  for (let index = 0; index < messages.length; index += 1) {
    const message = messages[index];
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

function compareNewestFirst(left, right) {
  return right.date_received.localeCompare(left.date_received);
}

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

function resolveMessage(mail, locator) {
  return resolveMessageInAccount(resolveAccount(mail, locator.account), locator);
}

function resolveMessageInAccount(account, locator) {
  const mailbox = resolveMailbox(account, locator.mailbox);
  const message = findMessage(mailbox, locator.id);
  if (!message) {
    fail("message_not_found", "No message matches the supplied locator");
  }
  return message;
}

function findMessage(mailbox, id) {
  const matches = mailbox.messages.whose({ id: id })();
  return matches.length === 0 ? null : matches[0];
}

function resolveAccount(mail, id) {
  const accounts = mail.accounts();
  for (let index = 0; index < accounts.length; index += 1) {
    if (String(accounts[index].id()) === id) {
      return accounts[index];
    }
  }
  fail("account_not_found", "No Mail account matches the supplied identifier");
}

function resolveMailbox(account, path) {
  const segments = decodeMailboxPath(path);
  let candidates = account.mailboxes();
  let current = null;
  for (let depth = 0; depth < segments.length; depth += 1) {
    current = findMailboxByName(candidates, segments[depth]);
    if (!current) {
      fail("mailbox_not_found", "No mailbox matches the supplied path");
    }
    candidates = current.mailboxes();
  }
  return current;
}

function findMailboxByName(mailboxes, name) {
  for (let index = 0; index < mailboxes.length; index += 1) {
    if (String(mailboxes[index].name()) === name) {
      return mailboxes[index];
    }
  }
  return null;
}

function mailboxPath(mailbox) {
  const segments = [];
  let current = mailbox;
  for (let depth = 0; depth <= 32; depth += 1) {
    segments.unshift(String(current.name()));
    let parent;
    try {
      parent = current.container();
      if (!parent || typeof parent.name !== "function") {
        return encodeMailboxPath(segments);
      }
      parent.name();
    } catch (_) {
      return encodeMailboxPath(segments);
    }
    current = parent;
  }
  fail("mailbox_depth_exceeded", "Mailbox nesting exceeds the supported depth");
}

function encodeMailboxPath(segments) {
  const path = "/" + segments.map(escapePathSegment).join("/");
  if (path.length > 4_096) {
    fail("mailbox_path_exceeded", "Mailbox path exceeds the supported limit");
  }
  return path;
}

function decodeMailboxPath(path) {
  requireText(path, "mailbox", 4_096);
  const encoded = path[0] === "/" ? path.slice(1).split("/") : path.split("/");
  if (
    encoded.length === 0 ||
    encoded.some(function (segment) {
      return segment.length === 0;
    })
  ) {
    fail("invalid_request", "Mailbox path must contain non-empty segments");
  }
  return encoded.map(unescapePathSegment);
}

function escapePathSegment(segment) {
  return segment.replace(/~/g, "~0").replace(/\//g, "~1");
}

function unescapePathSegment(segment) {
  if (/~(?:[^01]|$)/.test(segment)) {
    fail("invalid_request", "Mailbox path contains an invalid escape sequence");
  }
  return segment.replace(/~1/g, "/").replace(/~0/g, "~");
}

function requireLocator(request) {
  return {
    account: requireText(request.account, "account", 1_024),
    mailbox: requireText(request.mailbox, "mailbox", 4_096),
    id: requireInteger(request.id, "id", 0, Number.MAX_SAFE_INTEGER),
  };
}

function requireText(value, field, maxLength) {
  if (typeof value !== "string" || value.length === 0 || value.length > maxLength) {
    fail("invalid_request", field + " must be a non-empty bounded string");
  }
  return value;
}

function optionalText(value, field, maxLength) {
  return value === undefined ? null : requireText(value, field, maxLength);
}

function requireInteger(value, field, minimum, maximum) {
  if (!Number.isSafeInteger(value) || value < minimum || value > maximum) {
    fail("invalid_request", field + " must be an integer in the supported range");
  }
  return value;
}

function isoDate(value) {
  try {
    return boundedString(value.toISOString(), 128);
  } catch (_) {
    return boundedString(value, 128);
  }
}

function boundedString(value, maxLength) {
  const text = String(value);
  return text.length > maxLength ? text.slice(0, maxLength) : text;
}

function truncateUtf8(value, maxBytes) {
  let bytes = 0;
  let end = 0;
  while (end < value.length) {
    const first = value.charCodeAt(end);
    let width;
    let codeUnits = 1;
    if (first < 0x80) {
      width = 1;
    } else if (first < 0x800) {
      width = 2;
    } else if (first >= 0xd800 && first <= 0xdbff && end + 1 < value.length) {
      width = 4;
      codeUnits = 2;
    } else {
      width = 3;
    }
    if (bytes + width > maxBytes) {
      break;
    }
    bytes += width;
    end += codeUnits;
  }
  return { text: value.slice(0, end), isTruncated: end < value.length };
}

function fail(code, message) {
  const error = new Error(message);
  error.bridgeCode = code;
  throw error;
}

function errorEnvelope(error) {
  if (error && error.bridgeCode) {
    return {
      ok: false,
      error: { code: String(error.bridgeCode), message: String(error.message) },
    };
  }
  const number = error && typeof error.errorNumber === "number" ? error.errorNumber : null;
  if (number === -1743) {
    return {
      ok: false,
      error: {
        code: "permission_denied",
        message:
          "Allow this terminal to control Mail in System Settings > Privacy & Security > Automation",
      },
    };
  }
  if (number === -1712) {
    return {
      ok: false,
      error: { code: "mail_timeout", message: "Mail did not answer the Apple Event in time" },
    };
  }
  return {
    ok: false,
    error: {
      code: "mail_error",
      message: number === null ? "Mail operation failed" : "Mail operation failed (" + number + ")",
    },
  };
}
