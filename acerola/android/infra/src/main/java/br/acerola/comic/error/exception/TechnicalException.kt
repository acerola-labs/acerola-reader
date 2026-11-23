package br.acerola.comic.error.exception

/**
 * Technical exceptions that indicate BUGS or impossible states.
 * They must not be caught for business logic.
 * They are meant only for logging/Crashlytics.
 *
 * @param message Detailed message about the cause of the exception.
 * @param cause Root cause of the exception, if any.
 */
sealed class TechnicalException(
    message: String,
    cause: Throwable? = null,
) : RuntimeException(message, cause) {
    /**
     * Indicates an invalid state that should never happen in correct code.
     * E.g.: a service not initialized before use, an invalid internal configuration.
     *
     * @param message Detailed message about the invalid state.
     * @param cause Root cause of the exception.
     */
    class InvalidStateException(
        message: String,
        cause: Throwable? = null,
    ) : TechnicalException(message, cause)

    /**
     * Indicates a contract violation, usually related to invalid arguments passed to a method.
     * E.g.: a negative ID where only positive IDs are expected, an empty string where content is mandatory.
     *
     * @param message Detailed message about the contract violation.
     * @param cause Root cause of the exception.
     */
    class ContractViolationException(
        message: String,
        cause: Throwable? = null,
    ) : TechnicalException(message, cause)

    /**
     * Signals that an operation is not supported or not allowed in the current context.
     * E.g.: trying to perform a write operation on a read-only object.
     *
     * @param message Detailed message about the unsupported operation.
     * @param cause Root cause of the exception.
     */
    class UnsupportedOperationException(
        message: String,
        cause: Throwable? = null,
    ) : TechnicalException(message, cause)
}

/**
 * Helps enforce preconditions, throwing [ContractViolationException] if the condition is false.
 *
 * @param value Boolean condition that must be true.
 * @param lazyMessage Function that returns the error message if the condition is false.
 */
inline fun require(
    value: Boolean,
    lazyMessage: () -> String,
) {
    if (!value) {
        throw TechnicalException.ContractViolationException(lazyMessage())
    }
}

/**
 * Helps ensure a value is not null, throwing [InvalidStateException] if it is null.
 *
 * @param value The value to check.
 * @param lazyMessage Function that returns the error message if the value is null.
 * @return The non-null value.
 */
inline fun <T : Any> requireNotNull(
    value: T?,
    lazyMessage: () -> String,
): T = value ?: throw TechnicalException.InvalidStateException(lazyMessage())
