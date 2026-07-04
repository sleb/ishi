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

/// What `tk new`/`tk daily` create — a different vocabulary from
/// `Category` (where an item is filed). See design.md's "Filing
/// vocabulary vs. creation vocabulary" for why these are two types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Inbox,
    Project,
    Area,
    Resource,
    Daily,
}

impl Kind {
    /// The `Category` this kind files into. `Daily` maps to `Inbox` — a
    /// daily note has no folder of its own.
    pub fn category(&self) -> Category {
        match self {
            Kind::Inbox | Kind::Daily => Category::Inbox,
            Kind::Project => Category::Project,
            Kind::Area => Category::Area,
            Kind::Resource => Category::Resource,
        }
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

    #[test]
    fn kind_category_maps_correctly() {
        assert_eq!(Kind::Inbox.category(), Category::Inbox);
        assert_eq!(Kind::Project.category(), Category::Project);
        assert_eq!(Kind::Area.category(), Category::Area);
        assert_eq!(Kind::Resource.category(), Category::Resource);
        assert_eq!(Kind::Daily.category(), Category::Inbox);
    }
}
