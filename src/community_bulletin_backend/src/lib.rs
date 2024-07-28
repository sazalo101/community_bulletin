use candid::{CandidType, Deserialize};
use ic_cdk::api::time;
use ic_cdk::export::Principal;
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Clone)]
struct Post {
    id: u64,
    author: Principal,
    title: String,
    content: String,
    category: Category,
    timestamp: u64,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
enum Category {
    Announcement,
    Event,
    Classified,
}

#[derive(Default)]
struct BulletinBoardState {
    posts: HashMap<u64, Post>,
    post_counter: u64,
}

thread_local! {
    static STATE: RefCell<BulletinBoardState> = RefCell::new(BulletinBoardState::default());
}

/// Initialize the canister and set up the initial state and welcome post.

#[init]
fn init() {
    ic_cdk::println!("Canister initialized at timestamp: {}", time());
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        // Example: Initialize with a welcome post
        let welcome_post = Post {
            id: state.post_counter,
            author: ic_cdk::caller(),
            title: "Welcome".to_string(),
            content: "Welcome to the bulletin board!".to_string(),
            category: Category::Announcement,
            timestamp: time(),
        };
        state.posts.insert(state.post_counter, welcome_post);
        state.post_counter += 1;
    });
    ic_cdk::println!("Initial state setup complete.");
}

#[update]
fn create_post(title: String, content: String, category: Category) -> Result<u64, String> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let post_id = state.post_counter;
        state.post_counter += 1;

        let post = Post {
            id: post_id,
            author: ic_cdk::caller(),
            title,
            content,
            category,
            timestamp: time(),
        };

        state.posts.insert(post_id, post);
        Ok(post_id)
    }).map_err(|e| format!("Failed to create post: {}", e))
}

#[query]
fn get_post(id: u64) -> Option<Post> {
    STATE.with(|state| state.borrow().posts.get(&id).cloned())
}

#[query]
fn get_all_posts() -> Vec<Post> {
    STATE.with(|state| state.borrow().posts.values().cloned().collect())
}

#[query]
fn get_posts_by_category(category: Category) -> Vec<Post> {
    STATE.with(|state| {
        state
            .borrow()
            .posts
            .values()
            .filter(|post| post.category == category)
            .cloned()
            .collect()
    })
}

#[update]
fn update_post(id: u64, title: Option<String>, content: Option<String>, category: Option<Category>) -> Result<(), String> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(post) = state.posts.get_mut(&id) {
            if post.author == ic_cdk::caller() {
                if let Some(new_title) = title {
                    post.title = new_title;
                }
                if let Some(new_content) = content {
                    post.content = new_content;
                }
                if let Some(new_category) = category {
                    post.category = new_category;
                }
                Ok(())
            } else {
                Err("Only the author can update the post".to_string())
            }
        } else {
            Err("Post not found".to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_post() {
        let title = "Test Post".to_string();
        let content = "This is a test post.".to_string();
        let category = Category::Announcement;

        let post_id = create_post(title.clone(), content.clone(), category.clone()).unwrap();
        let post = get_post(post_id).unwrap();

        assert_eq!(post.title, title);
        assert_eq!(post.content, content);
        assert_eq!(post.category, category);
    }
}

#[update]
fn delete_post(id: u64) -> Result<(), String> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(post) = state.posts.get(&id) {
            if post.author == ic_cdk::caller() {
                state.posts.remove(&id);
                Ok(())
            } else {
                Err("Only the author can delete the post".to_string())
            }
        } else {
            Err("Post not found".to_string())
        }
    })
}

// Candid export
#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
