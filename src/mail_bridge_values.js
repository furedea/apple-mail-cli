// @ts-check

/**
 * @param {BridgeRequest} request
 * @returns {MessageLocator}
 */
function requireLocator(request) {
  return {
    account: requireText(request.account, "account", 1_024),
    mailbox: requireText(request.mailbox, "mailbox", 4_096),
    id: requireInteger(request.id, "id", { minimum: 0, maximum: Number.MAX_SAFE_INTEGER }),
  };
}

/**
 * @param {unknown} value
 * @param {string} field
 * @param {number} maxLength
 * @returns {string}
 */
function requireText(value, field, maxLength) {
  if (typeof value !== "string" || value.length === 0 || value.length > maxLength) {
    fail("invalid_request", field + " must be a non-empty bounded string");
  }
  return value;
}

/**
 * @param {unknown} value
 * @param {string} field
 * @param {number} maxLength
 * @returns {string | null}
 */
function optionalText(value, field, maxLength) {
  return value === undefined ? null : requireText(value, field, maxLength);
}

/**
 * @param {unknown} value
 * @param {string} field
 * @param {IntegerRange} range
 * @returns {number}
 */
function requireInteger(value, field, range) {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value < range.minimum ||
    value > range.maximum
  ) {
    fail("invalid_request", field + " must be an integer in the supported range");
  }
  return value;
}

/**
 * @param {unknown} value
 * @param {string} field
 * @returns {boolean}
 */
function requireBoolean(value, field) {
  if (typeof value !== "boolean") {
    fail("invalid_request", field + " must be a boolean");
  }
  return value;
}

/**
 * @param {JxaValue} value
 * @returns {string}
 */
function isoDate(value) {
  try {
    const date = /** @type {{toISOString: () => unknown}} */ (value);
    return boundedString(date.toISOString(), 128);
  } catch {
    return boundedString(value, 128);
  }
}

/**
 * @param {unknown} value
 * @param {number} maxLength
 * @returns {string}
 */
function boundedString(value, maxLength) {
  const text = String(value);
  return text.length > maxLength ? text.slice(0, maxLength) : text;
}

/**
 * @param {string} value
 * @param {number} maxBytes
 * @returns {TruncatedText}
 */
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

/**
 * @param {string} code
 * @param {string} message
 * @returns {never}
 */
function fail(code, message) {
  const error = /** @type {Error & {bridgeCode?: string}} */ (new Error(message));
  error.bridgeCode = code;
  throw error;
}

/**
 * @param {unknown} error
 * @returns {Record<string, unknown>}
 */
function errorEnvelope(error) {
  const failure =
    error && typeof error === "object" ? /** @type {BridgeErrorLike} */ (error) : null;
  if (failure && failure.bridgeCode) {
    return {
      ok: false,
      error: { code: String(failure.bridgeCode), message: String(failure.message) },
    };
  }
  const number = failure && typeof failure.errorNumber === "number" ? failure.errorNumber : null;
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
