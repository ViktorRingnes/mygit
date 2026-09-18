use std::path::Path;

use git2::{ObjectType, Odb, Oid, Repository, Tree as GitTree};
use serde::Serialize;
use utoipa::ToSchema;

use super::GitService;
use super::error::GitError;
use super::rev;

pub const MAX_DEPTH: u32 = 256;
pub const MAX_SCAN: usize = 100_000;

const SYMLINK: i32 = 0o120_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Directory,
    File,
    Symlink,
    Submodule,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub oid: String,
    #[schema(required)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Tree {
    pub rev: String,
    pub commit: String,
    pub path: String,
    pub entries: Vec<Entry>,
    pub offset: u64,
    #[schema(required)]
    pub total: Option<u64>,
    pub has_more: bool,
}

pub struct Request {
    pub rev: Option<String>,
    pub path: String,
    pub depth: u32,
    pub offset: u64,
    pub limit: u32,
}

impl GitService {
    pub async fn tree(&self, request: Request) -> Result<Tree, GitError> {
        self.read(move |repo| tree(repo, &request)).await
    }
}

fn tree(repo: &Repository, request: &Request) -> Result<Tree, GitError> {
    let revision = rev::resolve(repo, request.rev.as_deref())?;
    let root = revision.commit.tree()?;

    let target = if request.path.is_empty() {
        root
    } else {
        subtree(repo, &root, &request.path)?
    };

    let depth = match request.depth {
        0 => MAX_DEPTH,
        depth => depth.min(MAX_DEPTH),
    };

    let mut all = Vec::new();
    let capped = collect(repo, &repo.odb()?, &target, &request.path, depth, &mut all)?;

    let offset = usize::try_from(request.offset)
        .unwrap_or(usize::MAX)
        .min(all.len());
    let end = offset.saturating_add(request.limit as usize).min(all.len());

    Ok(Tree {
        rev: revision.name,
        commit: revision.commit.id().to_string(),
        path: request.path.clone(),
        has_more: capped || end < all.len(),
        total: if capped {
            None
        } else {
            u64::try_from(all.len()).ok()
        },
        entries: all.drain(offset..end).collect(),
        offset: request.offset,
    })
}

fn subtree<'repo>(
    repo: &'repo Repository,
    root: &GitTree<'repo>,
    path: &str,
) -> Result<GitTree<'repo>, GitError> {
    let entry = root
        .get_path(Path::new(path))
        .map_err(|_| GitError::NotFound(format!("path '{path}' does not exist")))?;

    entry
        .to_object(repo)?
        .into_tree()
        .map_err(|_| GitError::Invalid(format!("path '{path}' is not a directory")))
}

fn collect(
    repo: &Repository,
    odb: &Odb<'_>,
    tree: &GitTree<'_>,
    base: &str,
    depth: u32,
    out: &mut Vec<Entry>,
) -> Result<bool, GitError> {
    let mut level = Vec::with_capacity(tree.len());

    for item in tree {
        let name = item.name()?;

        let kind = kind(item.kind(), item.filemode());
        let oid = item.id();

        level.push((
            Entry {
                name: name.to_owned(),
                path: join(base, name),
                kind,
                oid: oid.to_string(),
                size: if kind == EntryKind::File {
                    size(odb, oid)
                } else {
                    None
                },
            },
            (kind == EntryKind::Directory).then_some(oid),
        ));
    }

    level.sort_by(|(a, _), (b, _)| order(a).cmp(&order(b)).then_with(|| a.name.cmp(&b.name)));

    for (entry, subtree) in level {
        if out.len() >= MAX_SCAN {
            return Ok(true);
        }

        let path = entry.path.clone();
        out.push(entry);

        if let Some(oid) = subtree
            && depth > 1
            && collect(repo, odb, &repo.find_tree(oid)?, &path, depth - 1, out)?
        {
            return Ok(true);
        }
    }

    Ok(false)
}

fn kind(object: Option<ObjectType>, filemode: i32) -> EntryKind {
    match object {
        Some(ObjectType::Tree) => EntryKind::Directory,
        Some(ObjectType::Commit) => EntryKind::Submodule,
        _ if filemode == SYMLINK => EntryKind::Symlink,
        _ => EntryKind::File,
    }
}

fn order(entry: &Entry) -> u8 {
    u8::from(entry.kind != EntryKind::Directory)
}

fn size(odb: &Odb<'_>, oid: Oid) -> Option<u64> {
    odb.read_header(oid)
        .ok()
        .and_then(|(size, _)| u64::try_from(size).ok())
}

fn join(base: &str, name: &str) -> String {
    if base.is_empty() {
        name.to_owned()
    } else {
        format!("{base}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::super::error::GitError;
    use super::super::fixture::Fixture;
    use super::{EntryKind, Request, Tree, tree};

    fn fixture() -> Fixture {
        let fixture = Fixture::new("main");

        fixture.commit(
            "init",
            &[
                ("readme.md", b"hello"),
                ("zeta.txt", b"z"),
                ("src/lib.rs", b"fn lib() {}"),
                ("src/nested/deep.rs", b"fn deep() {}"),
                ("docs/guide.md", b"guide"),
            ],
        );

        fixture
    }

    fn request(path: &str, depth: u32) -> Request {
        Request {
            rev: None,
            path: path.to_owned(),
            depth,
            offset: 0,
            limit: 1_000,
        }
    }

    #[test]
    fn lists_one_level_with_directories_first() {
        let fixture = fixture();

        let Tree { entries, total, .. } = tree(&fixture.repo, &request("", 1)).unwrap();

        let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();

        assert_eq!(names, ["docs", "src", "readme.md", "zeta.txt"]);
        assert_eq!(entries[0].kind, EntryKind::Directory);
        assert_eq!(entries[2].kind, EntryKind::File);
        assert_eq!(total, Some(4));
    }

    #[test]
    fn reports_blob_sizes_and_paths() {
        let fixture = fixture();

        let Tree { entries, .. } = tree(&fixture.repo, &request("src", 1)).unwrap();

        let lib = entries.iter().find(|entry| entry.name == "lib.rs").unwrap();

        assert_eq!(lib.path, "src/lib.rs");
        assert_eq!(lib.size, Some(11));
        assert_eq!(entries[0].path, "src/nested");
        assert_eq!(entries[0].size, None);
    }

    #[test]
    fn descends_the_requested_number_of_levels() {
        let fixture = fixture();

        let shallow = tree(&fixture.repo, &request("", 1)).unwrap();
        let two = tree(&fixture.repo, &request("", 2)).unwrap();
        let full = tree(&fixture.repo, &request("", 0)).unwrap();

        let paths = |tree: &Tree| -> Vec<String> {
            tree.entries
                .iter()
                .map(|entry| entry.path.clone())
                .collect()
        };

        assert_eq!(shallow.entries.len(), 4);
        assert!(!paths(&two).contains(&"src/nested/deep.rs".to_owned()));
        assert!(paths(&two).contains(&"src/nested".to_owned()));
        assert_eq!(
            paths(&full),
            [
                "docs",
                "docs/guide.md",
                "src",
                "src/nested",
                "src/nested/deep.rs",
                "src/lib.rs",
                "readme.md",
                "zeta.txt",
            ]
        );
    }

    #[test]
    fn pages_through_entries() {
        let fixture = fixture();

        let page = tree(
            &fixture.repo,
            &Request {
                offset: 1,
                limit: 2,
                ..request("", 1)
            },
        )
        .unwrap();

        let names: Vec<&str> = page
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();

        assert_eq!(names, ["src", "readme.md"]);
        assert_eq!(page.offset, 1);
        assert_eq!(page.total, Some(4));
        assert!(page.has_more);

        let last = tree(
            &fixture.repo,
            &Request {
                offset: 3,
                limit: 2,
                ..request("", 1)
            },
        )
        .unwrap();

        assert_eq!(last.entries.len(), 1);
        assert!(!last.has_more);
    }

    #[test]
    fn resolves_the_default_branch_and_reports_it() {
        let fixture = fixture();

        let listing = tree(&fixture.repo, &request("", 1)).unwrap();

        assert_eq!(listing.rev, "main");
        assert_eq!(listing.commit.len(), 40);
    }

    #[cfg(unix)]
    #[test]
    fn recognises_symlinks() {
        let fixture = Fixture::new("main");
        fixture.commit("init", &[("target.txt", b"x")]);
        fixture.symlink("link.txt", "target.txt");
        fixture.commit("add link", &[]);

        let Tree { entries, .. } = tree(&fixture.repo, &request("", 1)).unwrap();

        let link = entries
            .iter()
            .find(|entry| entry.name == "link.txt")
            .unwrap();

        assert_eq!(link.kind, EntryKind::Symlink);
        assert_eq!(link.size, None);
    }

    #[test]
    fn rejects_a_missing_directory() {
        let fixture = fixture();

        let error = tree(&fixture.repo, &request("nope", 1)).unwrap_err();

        assert!(matches!(error, GitError::NotFound(_)));
    }

    #[test]
    fn rejects_a_file_as_a_directory() {
        let fixture = fixture();

        let error = tree(&fixture.repo, &request("readme.md", 1)).unwrap_err();

        assert!(matches!(error, GitError::Invalid(_)));
    }

    #[test]
    fn rejects_an_unknown_revision() {
        let fixture = fixture();

        let error = tree(
            &fixture.repo,
            &Request {
                rev: Some("ghost".to_owned()),
                ..request("", 1)
            },
        )
        .unwrap_err();

        assert!(matches!(error, GitError::NotFound(_)));
    }
}
