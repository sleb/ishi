/// Discriminants are explicit because `Category as usize` indexes
/// `Config::category_dirs` (see `Workspace::category_dir`) — reordering a
/// variant without updating its value here would silently break that
/// mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Inbox = 0,
    Project = 1,
    Area = 2,
    Resource = 3,
    Archive = 4,
}

impl Category {
    pub fn is_directory_style(&self) -> bool {
        matches!(self, Category::Project | Category::Area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_and_area_are_directory_style() {
        assert!(Category::Project.is_directory_style());
        assert!(Category::Area.is_directory_style());
    }

    #[test]
    fn inbox_resource_archive_are_not_directory_style() {
        assert!(!Category::Inbox.is_directory_style());
        assert!(!Category::Resource.is_directory_style());
        assert!(!Category::Archive.is_directory_style());
    }
}
