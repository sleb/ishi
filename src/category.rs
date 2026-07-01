#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Inbox,
    Project,
    Area,
    Resource,
    Archive,
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
