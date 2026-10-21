// @ts-check

/**
 * @param {MailApplication} mail
 * @param {MessageLocator} locator
 * @returns {MailMessage}
 */
function resolveMessage(mail, locator) {
  return resolveMessageInAccount(resolveAccount(mail, locator.account), locator);
}

/**
 * @param {MailAccount} account
 * @param {MessageLocator} locator
 * @returns {MailMessage}
 */
function resolveMessageInAccount(account, locator) {
  const mailbox = resolveMailbox(account, locator.mailbox);
  const message = findMessage(mailbox, locator.id);
  if (!message) {
    fail("message_not_found", "No message matches the supplied locator");
  }
  return message;
}

/**
 * @param {MailMailbox} mailbox
 * @param {number} id
 * @returns {MailMessage | null}
 */
function findMessage(mailbox, id) {
  const matches = mailbox.messages.whose({ id: id })();
  const message = matches[0];
  return message === undefined ? null : message;
}

/**
 * @param {MailMailbox} mailbox
 * @param {string | null} messageId
 * @returns {MailMessage | null}
 */
function findMessageByMessageId(mailbox, messageId) {
  if (!messageId) {
    return null;
  }
  const matches = mailbox.messages.whose({ messageId: messageId })();
  const message = matches[0];
  return message === undefined ? null : message;
}

/**
 * @param {MailMessage} message
 * @returns {string | null}
 */
function messageIdForVerification(message) {
  try {
    const value = message.messageId();
    const text = value === null || value === undefined ? "" : String(value);
    return text.length > 0 && text.length <= 2_048 ? text : null;
  } catch {
    return null;
  }
}

/**
 * @param {MailApplication} mail
 * @param {string} id
 * @returns {MailAccount}
 */
function resolveAccount(mail, id) {
  const accounts = mail.accounts();
  for (const account of accounts) {
    if (String(account.id()) === id) {
      return account;
    }
  }
  fail("account_not_found", "No Mail account matches the supplied identifier");
}

/**
 * @param {MailAccount} account
 * @param {string} path
 * @returns {MailMailbox}
 */
function resolveMailbox(account, path) {
  const segments = decodeMailboxPath(path);
  let candidates = account.mailboxes();
  let current = null;
  for (const segment of segments) {
    current = findMailboxByName(candidates, segment);
    if (!current) {
      fail("mailbox_not_found", "No mailbox matches the supplied path");
    }
    candidates = current.mailboxes();
  }
  return /** @type {MailMailbox} */ (current);
}

/**
 * @param {MailMailbox[]} mailboxes
 * @param {string} name
 * @returns {MailMailbox | null}
 */
function findMailboxByName(mailboxes, name) {
  for (const mailbox of mailboxes) {
    if (String(mailbox.name()) === name) {
      return mailbox;
    }
  }
  return null;
}

/**
 * @param {MailMailbox} mailbox
 * @returns {string}
 */
function mailboxPath(mailbox) {
  const accountId = String(mailbox.account().id());
  const segments = [];
  /** @type {MailContainer} */
  let current = mailbox;
  for (let depth = 0; depth <= 32; depth += 1) {
    if (isAccount(current, accountId)) {
      return encodeMailboxPath(segments);
    }
    segments.unshift(String(current.name()));
    const parent = readableParent(current);
    if (!parent) {
      return encodeMailboxPath(segments);
    }
    current = parent;
  }
  fail("mailbox_depth_exceeded", "Mailbox nesting exceeds the supported depth");
}

/**
 * @param {MailMailbox} mailbox
 * @returns {MailContainer | null}
 */
function readableParent(mailbox) {
  try {
    const parent = mailbox.container();
    if (!parent || typeof parent.name !== "function") {
      return null;
    }
    parent.name();
    return parent;
  } catch {
    return null;
  }
}

/**
 * @param {MailContainer} value
 * @param {string} accountId
 * @returns {value is MailAccount}
 */
function isAccount(value, accountId) {
  try {
    return typeof value.id === "function" && String(value.id()) === accountId;
  } catch {
    return false;
  }
}

/**
 * @param {string[]} segments
 * @returns {string}
 */
function encodeMailboxPath(segments) {
  const path = "/" + segments.map(escapePathSegment).join("/");
  if (path.length > 4_096) {
    fail("mailbox_path_exceeded", "Mailbox path exceeds the supported limit");
  }
  return path;
}

/**
 * @param {string} path
 * @returns {string[]}
 */
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

/**
 * @param {string} segment
 * @returns {string}
 */
function escapePathSegment(segment) {
  return segment.replace(/~/g, "~0").replace(/\//g, "~1");
}

/**
 * @param {string} segment
 * @returns {string}
 */
function unescapePathSegment(segment) {
  if (/~(?:[^01]|$)/.test(segment)) {
    fail("invalid_request", "Mailbox path contains an invalid escape sequence");
  }
  return segment.replace(/~1/g, "/").replace(/~0/g, "~");
}
