use crate::operation::{RepositoryAdd, RepositoryRemove};

#[derive(Clone, Debug, PartialEq)]
pub enum SourceKind {
    Recommended { data: &'static [u8], enabled: bool },
    Custom,
}

#[derive(Clone, Debug)]
pub struct Source {
    pub backend_name: &'static str,
    pub id: String,
    pub name: String,
    pub kind: SourceKind,
    pub requires: Vec<String>,
}

impl Source {
    pub fn add(&self) -> Option<RepositoryAdd> {
        match self.kind {
            SourceKind::Recommended {
                data,
                enabled: false,
            } => Some(RepositoryAdd {
                id: self.id.clone(),
                data: data.to_vec(),
            }),
            _ => None,
        }
    }

    pub fn remove(&self) -> Option<RepositoryRemove> {
        match self.kind {
            SourceKind::Recommended { enabled: true, .. } | SourceKind::Custom => {
                Some(RepositoryRemove {
                    id: self.id.clone(),
                    name: self.name.clone(),
                })
            }
            _ => None,
        }
    }
}
