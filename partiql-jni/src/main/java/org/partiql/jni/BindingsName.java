package org.partiql.jni;

/**
 * Represents a name binding in PartiQL.
 * 
 * Names can be either:
 * - Delimited (case-sensitive): Requires exact match
 * - Undelimited (case-insensitive): Matches ignoring case
 * 
 * Matches the Rust BindingsName enum:
 * - CaseSensitive → delimited=true
 * - CaseInsensitive → delimited=false
 */
public final class BindingsName {
    private final String name;
    private final boolean delimited;
    
    /**
     * Create a new BindingsName.
     * 
     * @param name The name string
     * @param delimited true for case-sensitive (delimited), false for case-insensitive (undelimited)
     */
    public BindingsName(String name, boolean delimited) {
        if (name == null) {
            throw new IllegalArgumentException("Name cannot be null");
        }
        this.name = name;
        this.delimited = delimited;
    }
    
    /**
     * Returns the name string.
     */
    public String getName() {
        return name;
    }
    
    /**
     * Returns true if this name is delimited (case-sensitive).
     * Returns false if this name is undelimited (case-insensitive).
     */
    public boolean isDelimited() {
        return delimited;
    }
    
    /**
     * Create a delimited (case-sensitive) name.
     */
    public static BindingsName delimited(String name) {
        return new BindingsName(name, true);
    }
    
    /**
     * Create an undelimited (case-insensitive) name.
     */
    public static BindingsName undelimited(String name) {
        return new BindingsName(name, false);
    }
    
    @Override
    public String toString() {
        if (delimited) {
            return "\"" + name + "\"";  // Show delimited names in quotes
        } else {
            return name.toLowerCase();   // Show undelimited in lowercase
        }
    }
    
    @Override
    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (!(obj instanceof BindingsName)) return false;
        BindingsName other = (BindingsName) obj;
        
        // Must match on delimited flag
        if (this.delimited != other.delimited) return false;
        
        // Compare names based on case sensitivity
        if (delimited) {
            return this.name.equals(other.name);
        } else {
            return this.name.equalsIgnoreCase(other.name);
        }
    }
    
    @Override
    public int hashCode() {
        int result = delimited ? 1 : 0;
        result = 31 * result + (delimited ? name.hashCode() : name.toLowerCase().hashCode());
        return result;
    }
}
