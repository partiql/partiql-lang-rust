package org.partiql.jni;

/**
 * Represents a PartiQL value (ValueOwned in Rust).
 * This is a placeholder for the full value representation.
 * 
 * TODO: Implement full value type hierarchy matching partiql_value::Value
 */
public class Value {
    // For now, just store as Object
    // Future: Add proper type discrimination and accessors
    private final Object value;
    
    public Value(Object value) {
        this.value = value;
    }
    
    public Object getValue() {
        return value;
    }
    
    @Override
    public String toString() {
        return value != null ? value.toString() : "MISSING";
    }
}
