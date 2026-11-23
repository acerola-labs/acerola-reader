import { describe, expect, it } from 'vitest';
import { parseListKeys } from './r2';

describe('parseListKeys', () => {
	it('extracts every object key from a ListObjectsV2 response', () => {
		const xml = `<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult>
	<Name>acerola-reader</Name>
	<Contents><Key>android/latest/acerola-1.0.15.apk</Key><Size>1</Size></Contents>
	<Contents><Key>android/latest/acerola-1.2.0.apk</Key><Size>1</Size></Contents>
</ListBucketResult>`;

		expect(parseListKeys(xml)).toEqual([
			{ key: 'android/latest/acerola-1.0.15.apk' },
			{ key: 'android/latest/acerola-1.2.0.apk' }
		]);
	});

	it('returns no keys for an empty listing', () => {
		expect(
			parseListKeys('<ListBucketResult><KeyCount>0</KeyCount></ListBucketResult>')
		).toEqual([]);
	});

	it('unescapes XML entities in keys', () => {
		expect(parseListKeys('<Contents><Key>a&amp;b &lt;c&gt;</Key></Contents>')).toEqual([
			{ key: 'a&b <c>' }
		]);
	});
});
