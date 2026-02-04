package org.partiql.jni.exceptions;

/**
 * Thrown when an illegal state is detected, such as attempting to use
 * a closed VM or iterator.
 */
public class IllegalStateException extends PartiQLException {
    public IllegalStateException(String message) {
        super(message);
    }
}
