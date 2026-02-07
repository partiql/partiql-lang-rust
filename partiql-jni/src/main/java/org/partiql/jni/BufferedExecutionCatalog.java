package org.partiql.jni;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.function.Consumer;

/**
 * Execution catalog that stores all data in a pre-populated DirectByteBuffer.
 * 
 * This catalog eliminates Rust-to-Java callback overhead during query execution
 * by having users populate all data upfront at registration time. The data is
 * stored in a DirectByteBuffer that Rust can read directly without JNI calls.
 * 
 * <h2>Usage Example:</h2>
 * <pre>{@code
 * BufferedMemoryPool pool = new BufferedMemoryPool();
 * 
 * try (BufferedExecutionCatalog catalog1 = BufferedExecutionCatalog.create(entryId1, pool, writer -> {
 *     writer.putLong(0, 42);
 * })) {
 *     executionContext.addBufferedCatalog(catalogId, catalog1);
 *     // ... execute query ...
 * } // Buffer automatically returned to pool
 * 
 * try (BufferedExecutionCatalog catalog2 = BufferedExecutionCatalog.create(entryId2, pool, writer -> {
 *     writer.putLong(0, 43);
 * })) {
 *     // Reuses buffer from catalog1!
 *     executionContext.addBufferedCatalog(catalogId, catalog2);
 *     // ... execute query ...
 * }
 * }</pre>
 * 
 * <h2>Performance Benefits:</h2>
 * <ul>
 *   <li>Zero JNI callbacks during query execution</li>
 *   <li>DirectByteBuffer enables zero-copy memory access from Rust</li>
 *   <li>All data transfer happens once at registration time</li>
 *   <li>Buffer pooling amortizes DirectByteBuffer allocation costs</li>
 *   <li>Automatic buffer growth eliminates manual size calculations</li>
 *   <li>Buffer ID caching eliminates expensive GetDirectBufferAddress JNI calls</li>
 * </ul>
 * 
 * <h2>Limitations:</h2>
 * <ul>
 *   <li>All data must fit in memory at once</li>
 *   <li>Data is immutable after registration</li>
 *   <li>Currently supports single entry ID only (TODO: support multiple)</li>
 *   <li>Requires BufferedMemoryPool - non-pooled usage is not supported</li>
 * </ul>
 * 
 * <h2>Thread Safety:</h2>
 * <p>Instances are NOT thread-safe. The pool and catalogs should be used from a single thread.
 */
public final class BufferedExecutionCatalog implements AutoCloseable {
    private final long entryId;
    private ByteBuffer buffer;
    private final BufferedMemoryPool pool;
    
    /**
     * Private constructor. Use {@link #create} factory methods.
     * 
     * @param entryId Entry ID for this data source
     * @param buffer DirectByteBuffer containing the data
     * @param pool Memory pool for buffer management
     */
    private BufferedExecutionCatalog(long entryId, ByteBuffer buffer, BufferedMemoryPool pool) {
        this.entryId = entryId;
        this.buffer = buffer;
        this.pool = pool;
    }
    
    /**
     * Create a buffered catalog using a memory pool with default initial buffer size.
     * 
     * <p>The buffer will be returned to the pool when the catalog is closed. 
     * Use try-with-resources to ensure proper cleanup.
     * 
     * <p>The buffer will automatically grow as needed during population, eliminating
     * the need to calculate buffer sizes manually.
     * 
     * @param entryId Entry ID assigned during compilation
     * @param pool Memory pool for buffer management (required)
     * @param dataPopulator Consumer that writes data using BufferWriter
     * @return BufferedExecutionCatalog ready for registration (must be closed)
     */
    public static BufferedExecutionCatalog create(
        long entryId,
        BufferedMemoryPool pool,
        Consumer<BufferWriter> dataPopulator
    ) {
        if (pool == null) {
            throw new IllegalArgumentException("Pool cannot be null");
        }
        
        // Checkout buffer from pool
        ByteBuffer buffer = pool.checkout();
        
        // Create BufferWriter with pool reference for auto-growth
        BufferWriter writer = new BufferWriter(buffer, pool);
        
        // Let user populate
        dataPopulator.accept(writer);
        
        // Get final buffer (may have grown)
        buffer = writer.getBuffer();
        
        // Prepare buffer for reading by flipping it
        buffer.flip();
        
        return new BufferedExecutionCatalog(entryId, buffer, pool);
    }
    
    /**
     * Create a buffered catalog using a memory pool with specified initial buffer size.
     * 
     * <p>The buffer will be returned to the pool when the catalog is closed. 
     * Use try-with-resources to ensure proper cleanup.
     * 
     * <p>The buffer will automatically grow as needed during population, eliminating
     * the need to calculate buffer sizes manually. The specified bufferSize is only
     * a hint for the initial allocation.
     * 
     * @param entryId Entry ID assigned during compilation
     * @param pool Memory pool for buffer management (required)
     * @param bufferSize Initial buffer size hint in bytes
     * @param dataPopulator Consumer that writes data using BufferWriter
     * @return BufferedExecutionCatalog ready for registration (must be closed)
     */
    public static BufferedExecutionCatalog create(
        long entryId,
        BufferedMemoryPool pool,
        int bufferSize,
        Consumer<BufferWriter> dataPopulator
    ) {
        if (pool == null) {
            throw new IllegalArgumentException("Pool cannot be null");
        }
        if (bufferSize <= 0) {
            throw new IllegalArgumentException("Buffer size must be positive");
        }
        
        // Allocate buffer with custom size
        ByteBuffer buffer = ByteBuffer.allocateDirect(bufferSize)
                                      .order(ByteOrder.nativeOrder());
        
        // Create BufferWriter with pool reference for auto-growth
        BufferWriter writer = new BufferWriter(buffer, pool);
        
        // Let user populate
        dataPopulator.accept(writer);
        
        // Get final buffer (may have grown)
        buffer = writer.getBuffer();
        
        // Prepare buffer for reading by flipping it
        buffer.flip();
        
        return new BufferedExecutionCatalog(entryId, buffer, pool);
    }
    
    /**
     * Get the entry ID for this catalog.
     * Package-private for use by ExecutionContext.
     * 
     * @return Entry ID
     */
    long getEntryId() {
        return entryId;
    }
    
    /**
     * Get the data buffer.
     * Package-private for use by ExecutionContext.
     * 
     * @return DirectByteBuffer containing the data
     */
    ByteBuffer getBuffer() {
        return buffer;
    }
    
    /**
     * Get the memory pool.
     * Package-private for use by ExecutionContext.
     * 
     * @return BufferedMemoryPool managing this catalog's buffer
     */
    BufferedMemoryPool getPool() {
        return pool;
    }
    
    /**
     * Get the number of bytes written to the buffer.
     * 
     * @return Buffer size in bytes
     */
    public int getBufferSize() {
        return buffer.limit();
    }
    
    /**
     * Closes this catalog and returns its buffer to the pool (if pooled).
     * 
     * <p>After calling close(), the catalog should not be used. If the catalog
     * was created with a BufferedMemoryPool, the buffer will be returned to the
     * pool for reuse. Non-pooled catalogs can safely call close() with no effect.
     * 
     * <p>This method is idempotent - calling it multiple times is safe.
     */
    @Override
    public void close() {
        if (pool != null && buffer != null) {
            pool.returnBuffer(buffer);
            buffer = null;
        }
    }

    public void clear() {
        buffer.clear();
    }
}
