package org.partiql.jni.exceptions;

/**
 * Base exception for all PartiQL errors.
 * This is a checked exception.
 */
public class PartiQLException extends Exception {
    public PartiQLException(String message) {
        super(message);
    }
    
    public PartiQLException(String message, Throwable cause) {
        super(message, cause);
    }
}
