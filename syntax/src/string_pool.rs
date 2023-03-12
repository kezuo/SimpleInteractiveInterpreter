use std::collections::HashSet;

pub struct StringPool {
    strs: HashSet<Box<str>>,
}

impl StringPool {
    pub fn new() -> Self {
        StringPool {
            strs: HashSet::new(),
        }
    }

    unsafe fn cast_to_mut<T>(ptr: &T) -> &mut T {
        &mut *(ptr as *const T as *mut T)
    }

    pub fn get<'p>(&'p self, input: &str) -> &'p str {
        unsafe {
            let strs = Self::cast_to_mut(&self.strs);
            if !strs.contains(input) {
                let str = Box::from(input);
                strs.insert(str);
            }
            &*strs.get(input).unwrap()
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<str>> {
        self.strs.iter()
    }
}
