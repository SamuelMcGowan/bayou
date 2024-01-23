use std::ops::Range;

fn get_overlapping_groups<T, F: Fn(&T) -> Range<usize>>(
    mut ranges: Vec<T>,
    get_range: F,
) -> Vec<(Vec<T>, Range<usize>)> {
    /*
    - sort ranges by starts
    - go through ranges either adding the next range to the current group, or
      starting a new group
    */

    ranges.sort_by_key(|item| get_range(item).start);

    let mut groups = vec![];

    let mut group = vec![];
    let mut group_start = 0;
    let mut group_end = 0;

    for item in ranges {
        let range = get_range(&item);

        if range.start > group_end && !group.is_empty() {
            groups.push((std::mem::take(&mut group), group_start..group_end));
            group_start = range.start;
        }

        group_end = group_end.max(range.end);
        group.push(item);
    }

    if !group.is_empty() {
        groups.push((group, group_start..group_end));
    }

    groups
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::render2::get_overlapping_groups;

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn overlapping_ranges() {
        let ranges = vec![0..1, 0..10, 1..2, 5..7, 11..12];
        let overlapping_ranges = get_overlapping_groups(ranges, |r| r.clone());

        assert_eq!(
            &overlapping_ranges,
            &[
                (vec![0..1, 0..10, 1..2, 5..7], 0..10),
                (vec![11..12], 11..12)
            ]
        );
    }
}
