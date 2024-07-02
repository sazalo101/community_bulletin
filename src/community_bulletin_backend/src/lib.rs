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

#[init]
fn init() {
    ic_cdk::println!("Canister initialized");
}

#[update]
fn create_post(title: String, content: String, category: Category) -> u64 {
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
        post_id
    })
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
