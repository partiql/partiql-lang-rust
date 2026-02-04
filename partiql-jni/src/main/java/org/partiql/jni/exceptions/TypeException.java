package org.partiql.jni.exceptions;

/**
 * Thrown when a type mismatch occurs during query execution.
 */
public class TypeException extends PartiQLException {
    public TypeException(String message) {
        super(message);
    }
}
