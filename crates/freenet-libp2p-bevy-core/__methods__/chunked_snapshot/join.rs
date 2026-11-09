use crate::p2p;

pub fn join(
    chunks: &[Vec<u8>],
    manifest: &p2p::HistoryManifest,
) -> Result<Vec<u8>, p2p::ChunkError> {
    let actual_count =
        u32::try_from(chunks.len()).map_err(|e| p2p::ChunkError::Codec(e.to_string()))?;
    if actual_count != manifest.chunk_count {
        return Err(p2p::ChunkError::WrongChunkCount {
            expected: manifest.chunk_count,
            actual: chunks.len(),
        });
    }
    let compressed: Vec<u8> = chunks.iter().flatten().copied().collect();
    let actual = blake3::hash(&compressed).to_hex().to_string();
    if actual != *manifest.content_hash {
        return Err(p2p::ChunkError::HashMismatch);
    }
    zstd::decode_all(std::io::Cursor::new(compressed))
        .map_err(|e| p2p::ChunkError::Codec(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::join;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let split = p2p::ChunkedSnapshot::try_from_payload(&[7, 8, 9], 1).expect("split");
        let joined = join(&split.chunks, &split.manifest).expect("join");
        assert_eq!(joined, vec![7, 8, 9]);
    }

    #[test]
    fn test_usage_detects_corruption() {
        let split = p2p::ChunkedSnapshot::try_from_payload(&[7, 8, 9], 1).expect("split");
        let mut chunks = split.chunks.clone();
        if let Some(first) = chunks.first_mut() {
            first.push(0xFF);
        }
        assert!(join(&chunks, &split.manifest).is_err());
    }

    #[test]
    fn test_usage_rejects_wrong_count() {
        let split = p2p::ChunkedSnapshot::try_from_payload(&[7, 8, 9], 1).expect("split");
        let error = join(&[], &split.manifest).expect_err("empty");
        assert_eq!(
            error,
            p2p::ChunkError::WrongChunkCount {
                expected: split.manifest.chunk_count,
                actual: 0,
            }
        );
    }
}
