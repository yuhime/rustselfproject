# Merged Structs

A Rust procedural macro that lets you **merge two or more structs into a single standalone struct**.

The source structs declare their fields with `#[derive(MergedSource)]`. A destination struct can then use `#[merged(...)]` to collect fields from multiple source structs into one new struct.

Fields from merged source structs are automatically wrapped in `Option<T>`, allowing the resulting struct to represent data coming from multiple sources.

## Features

* Merge fields from two or more structs into one struct.
* Automatically converts merged fields to `Option<T>`.
* Keeps fields declared directly on the destination struct.
* Supports multiple source structs.
* Generates a standalone Rust struct rather than embedding the original structs.
* Automatically derives:

  * `Debug`
  * `serde::Serialize`
  * `serde::Deserialize`

## Example

Given two source structs:

```rust
use merged_structs::{merged, MergedSource};

#[derive(MergedSource)]
struct User {
    id: u64,
    username: String,
}

#[derive(MergedSource)]
struct Profile {
    email: String,
    age: u32,
}
```

You can merge them into a single struct:

```rust
#[merged(User, Profile)]
struct UserProfile {
    active: bool,
}
```

The generated struct is effectively:

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserProfile {
    pub id: Option<u64>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub age: Option<u32>,
    pub active: Option<bool>,
}
```

Notice that fields from the source structs become optional. This makes it possible to construct a merged struct even when only some of the source data is available.


}
```

produces:

```rust
pub active: Option<bool>,
```

## Multiple Sources

Any number of source structs can be supplied:

```rust
#[merged(User, Profile, Settings, Metadata)]
struct CompleteUser {
    source: String,
}
```

## Why?

This macro is useful when several structs represent different parts of the same conceptual data and you want to combine them into a single structure without manually duplicating their fields.

For example, you might have:

```text
User
 ├── id
 └── username

Profile
 ├── email
 └── avatar

Preferences
 ├── theme
 └── notifications
```

and want:

```text
CompleteUser
 ├── id
 ├── username
 ├── email
 ├── avatar
 ├── theme
 └── notifications
```

without manually maintaining the combined structure.
