export async function* chunkFile(file: File | undefined, size = 5_000_000) {
	if (!file) return;

	let offset = 0;

	while (offset < file.size) {
		const slice = file.slice(offset, offset + size);
		yield new Uint8Array(await slice.arrayBuffer());
		offset += size;
	}
}
