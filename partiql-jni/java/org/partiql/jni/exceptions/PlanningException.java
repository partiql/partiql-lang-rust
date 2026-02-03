package org.partiql.jni.exceptions;

/**
 * Thrown when query planning fails.
 */
public class PlanningException extends PartiQLException {
    public PlanningException(String message) {
        super(message);
    }
}
