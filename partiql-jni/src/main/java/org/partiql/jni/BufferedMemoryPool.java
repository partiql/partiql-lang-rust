package org.partiql.jni;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayDeque;
import java.util.IdentityHashMap;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * Memory pool for managing DirectByteBuffers used in BufferedExecutionCatalog.
 * 
 * <p>This pool amortizes the cost of DirectByteBuffer allocation across multiple
 * catalog instances by reusing buffers. It enables efficient buffer reuse patterns
 * where catalogs are created, used, and closed repeatedly.
 * 
 * <p>Design characteristics:
 * <ul>
 *   <li>Simple ArrayDeque-based pool (no synchronization needed - single threaded)</li>
 *   <li>Lazy allocation: buffers created on first checkout</li>
 *   <li>Bounded size: respects maxPoolSize to prevent unbounded growth</li>
 *   <li>Buffer growth: automatically provides larger buffers when needed</li>
 * </ul>
 * 
 * <p>Thread Safety: This class is NOT thread-safe. It is designed for single-threaded
 * use within a single execution context.
 * 
 * <p>Example usage:
 * <pre>{@code
 * BufferedMemoryPool pool = new BufferedMemoryPool();
 * 
 * try (BufferedExecutionCatalog catalog1 = BufferedExecutionCatalog.create(1L, pool, writer -> {
 *     writer.putLong(0, 42);
 * })) {
 *     // Use catalog1...
 * } // Buffer automatically returned to pool
 * 
 * try (BufferedExecutionCatalog catalog2 = BufferedExecutionCatalog.create(2L, pool, writer -> {
 *     writer.putLong(0, 43);
 * })) {
 *     // Reuses buffer from catalog1!
 * }
 * }</pre>
 */
public class BufferedMemoryPool {
    private final int initialBufferSize;
    private final int maxPoolSize;
    private final ArrayDeque<ByteBuffer> availableBuffers;
    private int totalPooledBuffers = 0;
    
    // Buffer ID tracking for Rust-side caching
    private final AtomicInteger nextBufferId = new AtomicInteger(1);
    private final IdentityHashMap<ByteBuffer, Integer> bufferIds = new IdentityHashMap<>();
    
    // Tracking for debugging/metrics
    private int totalCheckouts = 0;
    private int totalReturns = 0;
    private int totalAllocations = 0;
    private int totalGrowths = 0;
    
    /**
     * Creates a memory pool with default settings.
     * 
     * <p>Defaults:
     * <ul>
     *   <li>Initial buffer size: 1MB</li>
     *   <li>Max pool size: 4 buffers</li>
     * </ul>
     */
    public BufferedMemoryPool() {
        this(1024 * 1024, 4);
    }
    
    /**
     * Creates a memory pool with specified settings.
     * 
     * @param initialBufferSize Initial size of allocated buffers in bytes
     * @param maxPoolSize Maximum number of buffers to keep in pool
     */
    public BufferedMemoryPool(int initialBufferSize, int maxPoolSize) {
        if (initialBufferSize <= 0) {
            throw new IllegalArgumentException("initialBufferSize must be positive");
        }
        if (maxPoolSize <= 0) {
            throw new IllegalArgumentException("maxPoolSize must be positive");
        }
        
        this.initialBufferSize = initialBufferSize;
        this.maxPoolSize = maxPoolSize;
        this.availableBuffers = new ArrayDeque<>(maxPoolSize);
    }
    
    /**
     * Checks out a buffer from the pool.
     * 
     * <p>If a buffer is available in the pool, it will be reused. Otherwise,
     * a new DirectByteBuffer will be allocated. If the pool is at capacity,
     * a temporary buffer is allocated that won't be returned to the pool.
     * 
     * @return A DirectByteBuffer ready for use
     */
    ByteBuffer checkout() {
        totalCheckouts++;
        
        ByteBuffer buffer = availableBuffers.poll();
        if (buffer != null) {
            // Reuse pooled buffer
            buffer.clear();
            return buffer;
        }
        
        // Need to allocate a new buffer
        totalAllocations++;
        buffer = ByteBuffer.allocateDirect(initialBufferSize)
                          .order(ByteOrder.nativeOrder());
        
        if (totalPooledBuffers < maxPoolSize) {
            // Track that this buffer belongs to the pool
            totalPooledBuffers++;
        }
        // If totalPooledBuffers >= maxPoolSize, this is a temporary buffer
        // that won't be returned to the pool
        
        return buffer;
    }
    
    /**
     * Returns a buffer to the pool for reuse.
     * 
     * <p>The buffer will be cleared (position reset to 0, limit set to capacity)
     * before being placed back in the pool. If the pool is already at capacity,
     * the buffer will be discarded and allowed to be garbage collected.
     * 
     * @param buffer The buffer to return (may be null)
     */
    void returnBuffer(ByteBuffer buffer) {
        if (buffer == null) {
            return;
        }
        
        totalReturns++;
        
        // Only return to pool if we're not over capacity
        if (availableBuffers.size() < maxPoolSize) {
            buffer.clear();
            availableBuffers.offer(buffer);
        }
        // If over capacity, let the buffer be garbage collected
    }
    
    /**
     * Ensures a buffer has at least the required capacity.
     * 
     * <p>If the current buffer is large enough, it is returned as-is.
     * If not, a new larger buffer is allocated and existing data is copied.
     * The old buffer is NOT returned to the pool since it's the wrong size.
     * 
     * <p>The new buffer size is determined by doubling the current capacity
     * until it meets or exceeds the required size.
     * 
     * @param buffer Current buffer
     * @param requiredSize Minimum required capacity in bytes
     * @return A buffer with at least the required capacity
     */
    ByteBuffer ensureCapacity(ByteBuffer buffer, int requiredSize) {
        if (buffer.capacity() >= requiredSize) {
            return buffer;
        }
        
        totalGrowths++;
        
        // Calculate new size by doubling until we meet requirement
        int newSize = buffer.capacity();
        while (newSize < requiredSize) {
            newSize *= 2;
        }
        
        // Allocate new buffer
        totalAllocations++;
        ByteBuffer newBuffer = ByteBuffer.allocateDirect(newSize)
                                        .order(ByteOrder.nativeOrder());
        
        // Copy existing data
        int originalPosition = buffer.position();
        buffer.flip();
        newBuffer.put(buffer);
        
        // Don't return old buffer to pool (wrong size)
        // Just let it be garbage collected
        
        return newBuffer;
    }
    
    /**
     * Gets or assigns a unique ID for the given buffer.
     * 
     * <p>Each buffer gets a unique integer ID that remains stable throughout
     * its lifetime. This ID is used as a cache key in Rust to avoid repeated
     * JNI calls for buffer metadata (address and capacity).
     * 
     * @param buffer The buffer to get an ID for
     * @return Unique integer ID for this buffer
     */
    public int getBufferId(ByteBuffer buffer) {
        return bufferIds.computeIfAbsent(buffer, b -> nextBufferId.getAndIncrement());
    }
    
    /**
     * Clears the pool and releases all buffers.
     * 
     * <p>After calling clear(), the pool is empty and subsequent checkouts
     * will allocate new buffers.
     */
    public void clear() {
        availableBuffers.clear();
        bufferIds.clear();
        totalPooledBuffers = 0;
    }
    
    /**
     * Gets the initial buffer size used by this pool.
     * 
     * @return Initial buffer size in bytes
     */
    public int getInitialBufferSize() {
        return initialBufferSize;
    }
    
    /**
     * Gets the maximum pool size.
     * 
     * @return Maximum number of buffers that can be pooled
     */
    public int getMaxPoolSize() {
        return maxPoolSize;
    }
    
    /**
     * Gets the current number of available buffers in the pool.
     * 
     * @return Number of buffers currently available for checkout
     */
    public int getAvailableBufferCount() {
        return availableBuffers.size();
    }
    
    /**
     * Gets the total number of buffers that have been allocated for the pool.
     * 
     * @return Total buffers allocated (may be checked out or available)
     */
    public int getTotalPooledBuffers() {
        return totalPooledBuffers;
    }
    
    /**
     * Gets statistics about pool usage.
     * Useful for debugging and performance monitoring.
     * 
     * @return Pool statistics
     */
    public PoolStats getStats() {
        return new PoolStats(
            totalCheckouts,
            totalReturns,
            totalAllocations,
            totalGrowths,
            availableBuffers.size(),
            totalPooledBuffers
        );
    }
    
    /**
     * Statistics about memory pool usage.
     */
    public static class PoolStats {
        /** Total number of buffer checkouts */
        public final int totalCheckouts;
        
        /** Total number of buffer returns */
        public final int totalReturns;
        
        /** Total number of new buffer allocations */
        public final int totalAllocations;
        
        /** Total number of buffer growth operations */
        public final int totalGrowths;
        
        /** Current number of available buffers in pool */
        public final int availableBuffers;
        
        /** Total number of pooled buffers */
        public final int totalPooledBuffers;
        
        PoolStats(int checkouts, int returns, int allocations, int growths,
                  int available, int totalPooled) {
            this.totalCheckouts = checkouts;
            this.totalReturns = returns;
            this.totalAllocations = allocations;
            this.totalGrowths = growths;
            this.availableBuffers = available;
            this.totalPooledBuffers = totalPooled;
        }
        
        @Override
        public String toString() {
            return String.format(
                "PoolStats{checkouts=%d, returns=%d, allocations=%d, growths=%d, available=%d, totalPooled=%d}",
                totalCheckouts, totalReturns, totalAllocations, totalGrowths,
                availableBuffers, totalPooledBuffers
            );
        }
    }
}
