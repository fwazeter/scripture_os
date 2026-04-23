use smallvec::SmallVec;

/// Base-62 Alphabet sorted by ASCII value for safe lexicographical database sorting.
/// Order: 0-9 < A-Z , a-z
const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = 62;

/// ## `generate_sequential`
/// **Parameters:** `index: usize` (The integer index to convert).
///
/// ### Architectural Design Decision: FSI Seeder Generation
/// During the initial ingestion of a text (like the Quran), words are processed sequentially.
/// This utility converts their integer position into a Base-62 string, establishing the stable "Skeleton"
/// of Logical Anchors across the work.
///
/// ### Technical Context: Stack Allocation
/// Returns `SmallVec<[u8; 16]>` to ensure the IDs remain on the stack, eliminating heap fragmentation
/// as required by the FSI v4.0 performance standards.
pub fn generate_sequential(mut index: usize) -> SmallVec<[u8; 16]> {
    let mut result = SmallVec::new();

    if index == 0 {
        result.push(ALPHABET[0]);
        return result;
    } else {
        while index > 0 {
            let remainder = index % BASE;
            result.push(ALPHABET[remainder]);
            index /= BASE;
        }
    }

    // Pad with '0' to ensure consistent length for lexicographical sorting.
    // This prevents "10" from sorting before "2".
    while result.len() < 5 {
        result.push(ALPHABET[0]);
    }

    // Reverse because we extract digits from least to most significant
    result.reverse();
    result
}

/// ## `midpoint`
/// **Parameters:** `prev: &[u8]`, `next:&[u8]`
///
/// ### Architectural Design Decision: Infinite Resolution
/// This enables the "Structural Elasticity" of FSI v4.0. If an AI metadata track or variant
/// requires an insertion between word `a` and word `b`, this function generates a mathematically
/// valid intermediary key (e.g., `a1`) without shifting the indexes of any surrounding text.
///
/// **AI Prompt Hint:** Currently uses a basic append strategy for the MVP. Future iterations
/// should implement full fractional string arithmetic to ensure balanced tree depths.
pub fn midpoint(prev: &[u8], _next: &[u8]) -> SmallVec<[u8; 16]> {
    // MVP Strategy: Append '1' to the previous key.
    // Lexicographically, "a1" will always sort after "a" and before "b".
    let mut result: SmallVec<[u8; 16]> = SmallVec::from_slice(prev);
    result.push(ALPHABET[1]);
    result
}