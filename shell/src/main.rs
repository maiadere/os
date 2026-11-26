//! a filesystem browser
//!
//! shows the entries of the root directory and lets the buttons walk the tree: directories
//! expand in place, files get run. rendering goes through [`render`] alone, so moving it from
//! the console to a framebuffer later only touches that one function.

#![no_std]
#![no_main]

use gcapi::DirEntryRecord;
use heapless::{String, Vec};

// how many entries the browser can show at once
const MAX_ROWS: usize = 64;
// the longest path it can build, which is the longest the kernel accepts
const MAX_PATH: usize = 255;
// how many entries one list_dir call can return
const LIST_CAP: usize = 32;
// enough for a marker, the indent and a full length name
const MAX_LINE: usize = 256;

// gpio pins of the buttons, these have to be in the kernel's userspace::BUTTON_PINS to be read
const BTN_UP: u32 = 0;
const BTN_DOWN: u32 = 2;
const BTN_SELECT: u32 = 3;
const BTN_REFRESH: u32 = 25;

// how long to wait between polls of the buttons
const POLL_MS: u64 = 30;

/// one visible line of the browser
struct Row {
    /// the full path, which is what list_dir and load_program_from_path take
    path: String<MAX_PATH>,
    /// how many directories deep the entry is, only used for the indent
    depth: u8,
    is_dir: bool,
    /// whether this directory's children are currently in the list below it
    expanded: bool,
}

impl Row {
    /// the last component of the path, which is the entry's own name
    fn name(&self) -> &str {
        match self.path.rfind('/') {
            Some(slash) => &self.path[slash + 1..],
            None => &self.path,
        }
    }
}

/// joins a directory path and an entry name, avoiding a double slash at the root
fn join(parent: &str, name: &str) -> Option<String<MAX_PATH>> {
    let mut path: String<MAX_PATH> = String::new();
    path.push_str(parent).ok()?;
    if !parent.ends_with('/') {
        path.push('/').ok()?;
    }
    path.push_str(name).ok()?;
    Some(path)
}

/// drops `count` rows starting at `at`, undoing a listing that failed part way through
fn remove_rows(rows: &mut Vec<Row, MAX_ROWS>, at: usize, count: usize) {
    for _ in 0..count {
        if at < rows.len() {
            rows.remove(at);
        }
    }
}

/// lists `path` and inserts its entries into `rows` starting at index `at`, at depth `depth`
/// returns how many rows were inserted
/// on failure nothing is left behind, so the caller can retry without ending up with
/// the same entry listed twice
fn insert_entries(
    rows: &mut Vec<Row, MAX_ROWS>,
    at: usize,
    path: &str,
    depth: u8,
) -> Result<usize, ()> {
    let mut buffer = [DirEntryRecord::EMPTY; LIST_CAP];
    // the syscall reports the directory's total entry count, which is more than it wrote
    // whenever the buffer was too small
    let total = gcapi::list_dir(path, &mut buffer)?;
    let listed = total.min(LIST_CAP);

    for (offset, record) in buffer[..listed].iter().enumerate() {
        let Some(child) = join(path, record.name()) else {
            remove_rows(rows, at, offset);
            return Err(());
        };
        let row = Row {
            path: child,
            depth,
            is_dir: record.is_dir(),
            expanded: false,
        };
        if rows.insert(at + offset, row).is_err() {
            remove_rows(rows, at, offset);
            return Err(());
        }
    }

    if total > listed {
        gcapi::print("  (too many entries to show them all)\n");
    }

    Ok(listed)
}

/// drops every row below `at` that is deeper than it, which are exactly its descendants
fn collapse(rows: &mut Vec<Row, MAX_ROWS>, at: usize) {
    let depth = rows[at].depth;
    while rows.get(at + 1).is_some_and(|row| row.depth > depth) {
        rows.remove(at + 1);
    }
    rows[at].expanded = false;
}

/// acts on the selected row: a directory expands or collapses, a file gets run
fn select(rows: &mut Vec<Row, MAX_ROWS>, at: usize) {
    let Some(row) = rows.get(at) else {
        return;
    };
    // copied out so the borrow of rows ends before anything below mutates it
    let is_dir = row.is_dir;
    let expanded = row.expanded;
    let depth = row.depth;
    let path = row.path.clone();

    if !is_dir {
        let mut line: String<MAX_LINE> = String::new();
        let _ = line.push_str("running ");
        let _ = line.push_str(&path);
        let _ = line.push('\n');
        gcapi::print(&line);

        // this replaces the running program, so on success it never comes back here
        if gcapi::load_program_from_path(&path).is_err() {
            gcapi::print("could not run it\n");
        }
        return;
    }

    if expanded {
        collapse(rows, at);
        return;
    }

    if insert_entries(rows, at + 1, &path, depth + 1).is_err() {
        gcapi::print("could not list that directory\n");
        return;
    }
    rows[at].expanded = true;
}

/// prints the list, marking the selected row
/// the only thing here that draws, so graphics later means changing this and nothing else
fn render(rows: &Vec<Row, MAX_ROWS>, selected: usize) {
    gcapi::print("\n--- filesystem ---\n");

    if rows.is_empty() {
        gcapi::print("(empty)\n");
        return;
    }

    for (index, row) in rows.iter().enumerate() {
        // pushes are allowed to fail, an over long line is simply cut short
        let mut line: String<MAX_LINE> = String::new();
        let _ = line.push_str(if index == selected { ">" } else { " " });
        for _ in 0..row.depth {
            let _ = line.push_str("  ");
        }
        let _ = line.push(' ');
        let _ = line.push_str(row.name());
        if row.is_dir {
            let _ = line.push_str(if row.expanded { "/ [-]" } else { "/ [+]" });
        }
        let _ = line.push('\n');
        gcapi::print(&line);
    }
}

fn delay_ms(ms: u64) {
    let deadline = gcapi::get_time() + ms * 1000;
    while gcapi::get_time() < deadline {}
}

/// lists the root directory and hands it to the buttons
fn browse(rows: &mut Vec<Row, MAX_ROWS>) {
    let mut selected = 0usize;

    // "/" and "" both name the root, "." does not: a fat32 root has no "." entry
    if insert_entries(rows, 0, "/", 0).is_err() {
        gcapi::print("could not list the root directory\n");
        return;
    }
    render(rows, selected);

    // the syscall reports which buttons are held, so acting on the level would repeat for as
    // long as a button is down. only the bits that went from released to pressed count
    let mut previous = gcapi::read_buttons();

    loop {
        delay_ms(POLL_MS);

        let current = gcapi::read_buttons();
        let pressed = current & !previous;
        previous = current;

        if pressed == 0 {
            continue;
        }

        if pressed & (1u64 << BTN_UP) != 0 {
            selected = selected.saturating_sub(1);
        }
        if pressed & (1u64 << BTN_DOWN) != 0 && selected + 1 < rows.len() {
            selected += 1;
        }
        if pressed & (1u64 << BTN_REFRESH) != 0 {
            rows.clear();
            selected = 0;
            if insert_entries(rows, 0, "/", 0).is_err() {
                gcapi::print("could not list the root directory\n");
            }
        }
        if pressed & (1u64 << BTN_SELECT) != 0 {
            select(rows, selected);
        }

        // collapsing can leave the selection past the end of a now shorter list
        if selected >= rows.len() {
            selected = rows.len().saturating_sub(1);
        }

        render(rows, selected);
    }
}

#[unsafe(no_mangle)]
fn main() {
    // the rows live here rather than in a static, as a program's bss is not zeroed for it
    let mut rows: Vec<Row, MAX_ROWS> = Vec::new();
    browse(&mut rows);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    gcapi::print("shell panicked\n");
    gcapi::exit()
}
