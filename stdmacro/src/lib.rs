#![forbid(unsafe_code)]

#[macro_export]
macro_rules! deque {
    ($($elem:expr),*) => {{
            let mut deq = ::std::collections::VecDeque::with_capacity(1);
            $(deq.push_back($elem);)*
            deq
    }};
    ($elem:expr; $cap:literal) => {{
            let mut deq = ::std::collections::VecDeque::with_capacity($cap);
            deq.resize($cap, $elem);
            deq
    }};
}

#[macro_export]
macro_rules! sorted_vec {
    () => {
        Vec::new()
    };
    ($($elem:expr),*) => {{
        let mut vec = ::std::vec::Vec::new();
        $(vec.push($elem);)*
        vec.sort_unstable();
        vec
    }};
}

#[macro_export]
macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {{
            let mut map = ::std::collections::HashMap::new();
            $(map.insert($k, $v);)*
            map
    }};
}
