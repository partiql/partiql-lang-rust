package org.partiql.jni.exceptions;

/**
 * Thrown when a feature is not yet implemented.
 */
public class NotImplementedException extends PartiQLException {
    public NotImplementedException(String message) {
        super(message);
    }
}
