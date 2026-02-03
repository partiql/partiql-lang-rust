package org.partiql.jni;

/**
 * Handles loading of the native PartiQL library.
 * This is a stub - full implementation needed for JAR packaging.
 */
final class NativeLibrary {
    private static boolean loaded = false;
    
    /**
     * Ensures the native library is loaded. This is called automatically
     * by static initializers in classes that need native methods.
     */
    static synchronized void ensureLoaded() {
        if (loaded) {
            return;
        }
        
        // TODO: Implement proper library loading from JAR resources
        // For now, assume library is in java.library.path
        try {
            System.loadLibrary("partiql_jni");
            loaded = true;
        } catch (UnsatisfiedLinkError e) {
            throw new RuntimeException("Failed to load native library: partiql_jni", e);
        }
    }
}
