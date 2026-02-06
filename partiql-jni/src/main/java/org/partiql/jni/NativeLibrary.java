package org.partiql.jni;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * Handles loading of the native PartiQL library.
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
        
        String os = System.getProperty("os.name").toLowerCase();
        String libName;
        
        if (os.contains("mac")) {
            libName = "libpartiql_jni.dylib";
        } else if (os.contains("win")) {
            libName = "partiql_jni.dll";
        } else {
            libName = "libpartiql_jni.so";
        }
        
        try {
            // Try to extract from JAR and load
            extractAndLoad(libName);
            loaded = true;
        } catch (Exception e) {
            throw new RuntimeException("Failed to load native library: " + libName, e);
        }
    }
    
    private static void extractAndLoad(String libName) throws IOException {
        InputStream in = NativeLibrary.class.getResourceAsStream("/native/" + libName);
        
        if (in == null) {
            throw new IOException("Native library not found in JAR: " + libName);
        }
        
        // Extract to temp file
        Path tempFile = Files.createTempFile("partiql_jni", 
            libName.substring(libName.lastIndexOf('.')));
        tempFile.toFile().deleteOnExit();
        
        Files.copy(in, tempFile, StandardCopyOption.REPLACE_EXISTING);
        in.close();
        
        // Load from temp location
        System.load(tempFile.toAbsolutePath().toString());
    }
}
