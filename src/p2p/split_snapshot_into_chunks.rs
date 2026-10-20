use crate::p2p;

const CHUNK_LIMIT: usize = 60 * 1024;

pub fn split_snapshot_into_chunks(
    payload: &[u8],
    lamport_max: u64,
) -> Result<p2p::ChunkedSnapshot, p2p::ChunkError> {
    let compressed = zstd::encode_all(std::io::Cursor::new(payload), 0)
        .map_err(|e| p2p::ChunkError::Codec(e.to_string()))?;
    let content_hash = p2p::ContentHash::from(blake3::hash(&compressed).to_hex().to_string());
    let chunks: Vec<Vec<u8>> = compressed.chunks(CHUNK_LIMIT).map(<[u8]>::to_vec).collect();
    let chunk_count =
        u32::try_from(chunks.len().max(1)).map_err(|e| p2p::ChunkError::Codec(e.to_string()))?;
    let chunks = if chunks.is_empty() {
        vec![Vec::new()]
    } else {
        chunks
    };
    Ok(p2p::ChunkedSnapshot {
        manifest: p2p::HistoryManifest {
            lamport_max,
            chunk_count,
            content_hash,
        },
        chunks,
    })
}

#[cfg(test)]
mod tests {
    use super::split_snapshot_into_chunks;
    use crate::p2p;

    fn round_trips(size: usize) {
        let payload: Vec<u8> = (0..size)
            .map(|i| u8::try_from(i % 251).unwrap_or(u8::MAX))
            .collect();
        let split = split_snapshot_into_chunks(&payload, 3).expect("split");
        let expected = usize::try_from(split.manifest.chunk_count).unwrap_or(usize::MAX);
        assert_eq!(split.chunks.len(), expected);
        for chunk in &split.chunks {
            assert!(chunk.len() <= 60 * 1024);
        }
        let joined = p2p::join_chunks_into_snapshot(&split.chunks, &split.manifest).expect("join");
        assert_eq!(joined, payload);
    }

    #[test]
    fn test_usage() {
        round_trips(0);
        round_trips(1);
    }

    #[test]
    fn test_usage_boundary_sizes() {
        round_trips(59 * 1024);
        round_trips(61 * 1024);
    }

    #[test]
    fn test_usage_large_snapshot() {
        round_trips(5 * 1024 * 1024);
    }

    #[test]
    fn test_usage_content_hash_differs() {
        let first = split_snapshot_into_chunks(&[1, 2, 3], 0).expect("split");
        let second = split_snapshot_into_chunks(&[1, 2, 4], 0).expect("split");
        assert_ne!(first.manifest.content_hash, second.manifest.content_hash);
    }
}
