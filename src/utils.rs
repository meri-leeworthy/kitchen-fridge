///! Some utility functions

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use minidom::Element;

use crate::traits::CompleteCalendar;
use crate::calendar::CalendarId;

/// Walks an XML tree and returns every element that has the given name
pub fn find_elems<S: AsRef<str>>(root: &Element, searched_name: S) -> Vec<&Element> {
    let searched_name = searched_name.as_ref();
    let mut elems: Vec<&Element> = Vec::new();

    for el in root.children() {
        if el.name() == searched_name {
            elems.push(el);
        } else {
            let ret = find_elems(el, searched_name);
            elems.extend(ret);
        }
    }
    elems
}

/// Walks an XML tree until it finds an elements with the given name
pub fn find_elem<S: AsRef<str>>(root: &Element, searched_name: S) -> Option<&Element> {
    let searched_name = searched_name.as_ref();
    if root.name() == searched_name {
        return Some(root);
    }

    for el in root.children() {
        if el.name() == searched_name {
            return Some(el);
        } else {
            let ret = find_elem(el, searched_name);
            if ret.is_some() {
                return ret;
            }
        }
    }
    None
}

pub fn print_xml(element: &Element) {
    use std::io::Write;
    let mut writer = std::io::stdout();

    let mut xml_writer = minidom::quick_xml::Writer::new_with_indent(
        std::io::stdout(),
        0x20, 4
    );
    let _ = element.to_writer(&mut xml_writer);
    let _ = writer.write(&[0x0a]);
}

/// A debug utility that pretty-prints calendars
pub fn print_calendar_list<C>(cals: &HashMap<CalendarId, Arc<Mutex<C>>>)
where
    C: CompleteCalendar,
{
    for (id, cal) in cals {
        println!("CAL {}", id);
        for (_, item) in cal.lock().unwrap().get_items() {
            let task = item.unwrap_task();
            let completion = if task.completed() {"✓"} else {" "};
            println!("    {} {}\t{}", completion, task.name(), task.id());
        }
    }
}
