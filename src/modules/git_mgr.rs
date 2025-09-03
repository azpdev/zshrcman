use anyhow::{Context, Result};
use git2::{
    BranchType, Cred, FetchOptions, PushOptions, RemoteCallbacks, 
    Repository, ResetType, Signature
};
use std::path::Path;

pub struct GitManager {
    repo: Repository,
}

impl GitManager {
    pub fn init_or_clone(path: &Path, remote_url: Option<&str>) -> Result<Self> {
        let repo = if let Some(url) = remote_url {
            if path.exists() {
                Repository::open(path)?
            } else {
                Self::clone_repo(url, path)?
            }
        } else {
            Repository::init(path)?
        };
        
        Ok(Self { repo })
    }
    
    fn clone_repo(url: &str, path: &Path) -> Result<Repository> {
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        fetch_options.remote_callbacks(callbacks);
        
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);
        
        builder.clone(url, path)
            .context("Failed to clone repository")
    }
    
    pub fn list_remote_branches(&self) -> Result<Vec<String>> {
        let mut remote = self.repo.find_remote("origin")?;
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None)?;
        
        let refs = remote.list()?;
        let branches: Vec<String> = refs
            .iter()
            .filter_map(|r| {
                let name = r.name();
                if name.starts_with("refs/heads/") {
                    Some(name.strip_prefix("refs/heads/").unwrap().to_string())
                } else {
                    None
                }
            })
            .collect();
        
        remote.disconnect()?;
        Ok(branches)
    }
    
    pub fn checkout_branch(&self, branch: &str, create: bool) -> Result<()> {
        if create {
            let head = self.repo.head()?;
            let oid = head.target().context("No HEAD target")?;
            let commit = self.repo.find_commit(oid)?;
            
            self.repo.branch(branch, &commit, false)?;
        }
        
        let obj = self.repo.revparse_single(&format!("refs/heads/{}", branch))?;
        self.repo.checkout_tree(&obj, None)?;
        self.repo.set_head(&format!("refs/heads/{}", branch))?;
        
        Ok(())
    }
    
    pub fn fetch_and_pull(&self, branch: &str) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        fetch_options.remote_callbacks(callbacks);
        
        remote.fetch(&[branch], Some(&mut fetch_options), None)?;
        
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;
        
        if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", branch);
            let mut reference = self.repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            self.repo.set_head(&refname)?;
            self.repo.checkout_head(None)?;
        } else if analysis.0.is_normal() {
            let head_commit = self.repo.reference_to_annotated_commit(&self.repo.head()?)?;
            self.repo.merge(&[&fetch_commit], None, None)?;
            
            let signature = Signature::now("zshrcman", "zshrcman@localhost")?;
            let tree_id = self.repo.index()?.write_tree()?;
            let tree = self.repo.find_tree(tree_id)?;
            let parent_commit = self.repo.find_commit(head_commit.id())?;
            let fetch_commit_obj = self.repo.find_commit(fetch_commit.id())?;
            
            self.repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Merge from origin",
                &tree,
                &[&parent_commit, &fetch_commit_obj],
            )?;
        }
        
        Ok(())
    }
    
    pub fn commit_and_push(&self, message: &str, branch: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let signature = Signature::now("zshrcman", "zshrcman@localhost")?;
        
        let parent_commit = if let Ok(head) = self.repo.head() {
            let oid = head.target().context("No HEAD target")?;
            Some(self.repo.find_commit(oid)?)
        } else {
            None
        };
        
        let parent_commits = if let Some(ref parent) = parent_commit {
            vec![parent]
        } else {
            vec![]
        };
        
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_commits,
        )?;
        
        let mut remote = self.repo.find_remote("origin")?;
        let mut push_options = PushOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        push_options.remote_callbacks(callbacks);
        
        remote.push(&[&format!("refs/heads/{}", branch)], Some(&mut push_options))?;
        
        Ok(())
    }
    
    pub fn add_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(&["."], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }
    
    pub fn sync(&self, main_branch: &str, device_branch: &str) -> Result<()> {
        self.fetch_and_pull(main_branch)?;
        
        self.checkout_branch(main_branch, false)?;
        
        self.checkout_branch(device_branch, false)?;
        
        let main_ref = self.repo.revparse_single(&format!("refs/heads/{}", main_branch))?;
        let main_commit = main_ref.peel_to_commit()?;
        
        let mut rebase_opts = git2::RebaseOptions::new();
        let signature = Signature::now("zshrcman", "zshrcman@localhost")?;
        
        let annotated = self.repo.reference_to_annotated_commit(
            &self.repo.find_reference(&format!("refs/heads/{}", main_branch))?
        )?;
        
        let mut rebase = self.repo.rebase(None, Some(&annotated), None, Some(&mut rebase_opts))?;
        
        while let Some(_op) = rebase.next() {
            if let Err(e) = rebase.commit(None, &signature, None) {
                rebase.abort()?;
                return Err(anyhow::anyhow!("Rebase failed: {}", e));
            }
        }
        
        rebase.finish(Some(&signature))?;
        
        Ok(())
    }
}