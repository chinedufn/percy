use crate::diff::ElementKey;

const PLACEHOLDER_KEY_AND_IDX: KeyAndChildIdx = KeyAndChildIdx {
    key: ElementKey::Explicit("..."),
    child_idx: 123,
};
const PLACEHOLDER_USIZE: usize = 55555;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Debug))]
pub(super) struct KeyAndChildIdx<'a> {
    pub key: ElementKey<'a>,
    pub child_idx: usize,
}

pub(super) fn get_longest_increasing_subsequence<'a>(
    original: &'a [KeyAndChildIdx],
) -> Vec<KeyAndChildIdx<'a>> {
    if original.len() == 0 {
        return vec![];
    }

    let mut predecessor = vec![PLACEHOLDER_USIZE; original.len()];
    let mut m = vec![PLACEHOLDER_USIZE; original.len() + 1];

    let mut longest_length_found = 0;

    for idx in 0..original.len() {
        let mut low = 1;
        let mut high = longest_length_found + 1;

        while low < high {
            let mid = (high - low) / 2;
            let mid = low + mid;

            if original[m[mid]].child_idx > original[idx].child_idx {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        let new_longest_length_found = low;
        predecessor[idx] = m[new_longest_length_found - 1];
        m[new_longest_length_found] = idx;

        if new_longest_length_found > longest_length_found {
            longest_length_found = new_longest_length_found;
        }
    }

    let mut subsequence = vec![PLACEHOLDER_KEY_AND_IDX; longest_length_found];
    let mut k = m[longest_length_found];

    let mut loop_backwards = longest_length_found;
    while loop_backwards > 0 {
        loop_backwards = loop_backwards - 1;
        subsequence[loop_backwards] = original[k];
        k = predecessor[k];
    }

    subsequence
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: KeyAndChildIdx = make_val("a", 0);
    const B: KeyAndChildIdx = make_val("b", 1);
    const C: KeyAndChildIdx = make_val("c", 2);
    const D: KeyAndChildIdx = make_val("d", 3);
    const E: KeyAndChildIdx = make_val("e", 4);

    const fn make_val(key: &'static str, val: usize) -> KeyAndChildIdx {
        KeyAndChildIdx {
            key: ElementKey::Explicit(key),
            child_idx: val,
        }
    }

    /// Verify that we properly determine longest increasing subsequences.
    #[test]
    fn longest_increasing_subsequences() {
        for (idx, (full_sequence, expected_subsequence)) in vec![
            (vec![], vec![]),
            (vec![A, B, C, D, E], vec![A, B, C, D, E]),
            (vec![E, D, C, B, A], vec![A]),
            (vec![A, B, C, E, D], vec![A, B, C, D]),
            (vec![A, D, C, E], vec![A, C, E]),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                get_longest_increasing_subsequence(&full_sequence),
                expected_subsequence,
                "Test at index {} failed.",
                idx
            );
        }
    }
}
