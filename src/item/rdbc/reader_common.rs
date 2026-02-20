//! Common functionality for database item readers.
//!
//! This module provides shared utilities used across all database-specific
//! item readers (PostgreSQL, MySQL, SQLite) to reduce code duplication and ensure
//! consistent behavior.

/// Calculates the index within the current page based on offset and page size.
///
/// # Arguments
///
/// * `offset` - The current offset in the overall result set
/// * `page_size` - Optional page size for pagination
///
/// # Returns
///
/// The index within the current page. For paginated reading (page_size is Some),
/// returns `offset % page_size`. For non-paginated reading, returns the offset itself.
///
/// # Examples
///
/// ```ignore
/// // Paginated reading with page_size=10
/// assert_eq!(calculate_page_index(0, Some(10)), 0);
/// assert_eq!(calculate_page_index(5, Some(10)), 5);
/// assert_eq!(calculate_page_index(10, Some(10)), 0); // New page starts
/// assert_eq!(calculate_page_index(15, Some(10)), 5);
///
/// // Non-paginated reading
/// assert_eq!(calculate_page_index(0, None), 0);
/// assert_eq!(calculate_page_index(5, None), 5);
/// assert_eq!(calculate_page_index(100, None), 100);
/// ```
#[inline]
pub fn calculate_page_index(offset: i32, page_size: Option<i32>) -> i32 {
    if let Some(page_size) = page_size {
        offset % page_size
    } else {
        offset
    }
}

/// Checks if a new page needs to be loaded.
///
/// A new page should be loaded when the index within the current page is 0,
/// which indicates we're at the start of a new page or this is the first read.
///
/// # Arguments
///
/// * `page_index` - The current index within the page (from `calculate_page_index`)
///
/// # Returns
///
/// `true` if a new page should be loaded, `false` otherwise.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(should_load_page(0), true);  // Start of page, load data
/// assert_eq!(should_load_page(1), false); // Middle of page, use buffer
/// assert_eq!(should_load_page(5), false); // Middle of page, use buffer
/// ```
#[inline]
pub fn should_load_page(page_index: i32) -> bool {
    page_index == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_page_index_with_pagination() {
        assert_eq!(calculate_page_index(0, Some(10)), 0);
        assert_eq!(calculate_page_index(5, Some(10)), 5);
        assert_eq!(calculate_page_index(9, Some(10)), 9);
        assert_eq!(calculate_page_index(10, Some(10)), 0); // New page
        assert_eq!(calculate_page_index(15, Some(10)), 5);
        assert_eq!(calculate_page_index(20, Some(10)), 0); // Another new page
    }

    #[test]
    fn test_calculate_page_index_without_pagination() {
        assert_eq!(calculate_page_index(0, None), 0);
        assert_eq!(calculate_page_index(5, None), 5);
        assert_eq!(calculate_page_index(100, None), 100);
        assert_eq!(calculate_page_index(1000, None), 1000);
    }

    #[test]
    fn test_should_load_page() {
        assert!(should_load_page(0));
        assert!(!should_load_page(1));
        assert!(!should_load_page(5));
        assert!(!should_load_page(99));
    }
}
