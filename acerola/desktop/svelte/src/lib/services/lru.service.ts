import { LRUCache } from 'lru-cache';

export interface LRUConfig<
	KeyType extends string | number = string | number,
	ValueType extends {} = {}
> {
	/**
	 * Strict maximum number of items kept in memory.
	 */
	max: number;
	dispose?: (value: ValueType, key: KeyType) => void;
}

/**
 * A generic LRU (Least Recently Used) service abstraction.
 * Works strictly on maximum capacity, evicting the least recently used
 * item when the limit is exceeded.
 */
export class LRUService<KeyType extends string | number, ValueType extends {}> {
	private cache: LRUCache<KeyType, ValueType>;

	constructor(config: LRUConfig<KeyType, ValueType>) {
		this.cache = new LRUCache({
			max: config.max,
			dispose: (value, key) => {
				config.dispose?.(value, key);
				console.log(`[LRUService] 🗑️ EVICTING page ${key} (Cache limit reached)`);
			}
		});
	}

	/**
	 * Retrieves an item from the cache without updating its "recently used" status.
	 */
	peek(key: KeyType): ValueType | undefined {
		return this.cache.peek(key);
	}

	/**
	 * Retrieves an item from the cache and marks it as most recently used (MRU).
	 */
	get(key: KeyType): ValueType | undefined {
		return this.cache.get(key);
	}

	/**
	 * Sets an item in the cache and makes it the most recently used (MRU).
	 * If the limit is exceeded, the oldest LRU item is removed.
	 */
	set(key: KeyType, value: ValueType): void {
		const isNew = !this.cache.has(key);
		this.cache.set(key, value);
		console.log(
			`[LRUService] SET key=${key} | size=${this.cache.size}/${this.cache.max} | new=${isNew}`
		);
	}

	/**
	 * Checks whether an item exists in the cache without changing its recency.
	 */
	has(key: KeyType): boolean {
		return this.cache.has(key);
	}

	/**
	 * Returns the present keys, ordered internally by insertion recency (LRU order in lru-cache).
	 */
	get keys(): KeyType[] {
		return Array.from(this.cache.keys());
	}

	/**
	 * Returns all values in the cache.
	 */
	get values(): ValueType[] {
		return Array.from(this.cache.values());
	}

	/**
	 * Actively removes a key from the cache.
	 */
	delete(key: KeyType): void {
		console.log(`[LRUService] DELETE key=${key}`);
		this.cache.delete(key);
	}

	/**
	 * Clears the cache entirely.
	 */
	clear(): void {
		if (this.cache.size > 0) {
			console.log(`[LRUService] CLEAR cache (prev size: ${this.cache.size})`);
			this.cache.clear();
		}
	}

	get size(): number {
		return this.cache.size;
	}
}
