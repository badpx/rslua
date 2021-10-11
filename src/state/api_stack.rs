use crate::api::LuaAPI;
use super::lua_state::LuaState;

impl LuaAPI for LuaState {
    
fn get_top(&self) -> isize {
    self.stack.top()
}

fn abs_index(&self, idx: isize) -> isize {
    self.stack.abs_index(idx)
}

fn check_stack(&mut self, n: usize) -> bool {
    self.stack.check(n);
    return true;    // Never fails
}

/*
              pop(2)
    +-------+       +-------+
    |   d   |       |       |
    +-------+       +-------+
    |   c   |       |       |
    +-------+       +-------+
    |   b   |       |   b   |
    +-------+       +-------+
    |   a   | ====> |   a   |
    +-------+       +-------+
*/
fn pop(&mut self, n: usize) {
    for _ in 0..n {
        self.stack.pop();
    }
}

/*
               copy
    +-------+       +-------+
    |       |       |       |
    +-------+       +-------+
    |   c   |   +-->|   b   |
    +-------+   |   +-------+
    |   b   |---+   |   b   |
    +-------+       +-------+
    |   a   |       |   a   |
    +-------+       +-------+
*/
fn copy(&mut self, from: isize, to: isize) {
    let val = self.stack.get(from);
    self.stack.set(to, val);
}

/*
               push
    +-------+       +-------+
    |       |   +-->|   b   |
    +-------+   |   +-------+
    |   c   |   |   |   c   |
    +-------+   |   +-------+
    |   b   |---+   |   b   |
    +-------+       +-------+
    |   a   |       |   a   |
    +-------+       +-------+
*/
fn push_value(&mut self, idx: isize) {
    let val = self.stack.get(idx);
    self.stack.push(val);
}

/*
             replace
    +-------+       +-------+
    |   d   |---+   |       |
    +-------+   |   +-------+
    |   c   |   |   |   c   |
    +-------+   |   +-------+
    |   b   |   +-->|   d   |
    +-------+       +-------+
    |   a   |       |   a   |
    +-------+       +-------+
*/
fn replace(&mut self, idx: isize) {
    let val = self.stack.pop();
    self.stack.set(idx, val);
}

/*
              insert
    +-------+        +-------+
    |   d   |---+/-->|   c   |
    +-------+  /|    +-------+
    |   c   |/  |/-->|   b   |
    +-------+  /|    +-------+
    |   b   |/  +--->|   d   |
    +-------+        +-------+
    |   a   |        |   a   |
    +-------+        +-------+
*/
fn insert(&mut self, idx: isize) {
    self.rotate(idx, 1);
}

fn remove(&mut self, idx: isize) {
    self.rotate(idx, -1);
    self.pop(1);
}

fn rotate(&mut self, idx: isize, n: isize) {
    let t = self.stack.top() as usize - 1;
    let p = self.stack.abs_index(idx) as usize - 1;
    let m = if n >= 0 { t - n as usize } else { p - n as usize - 1 };
    self.stack.reverse(p, m);
    self.stack.reverse(m + 1, t);
    self.stack.reverse(p, t);
}
}