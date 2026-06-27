use taffy::prelude::*;

fn main() {
    let mut taffy = TaffyTree::new();

    let spacer = taffy.new_leaf(Style {
        flex_grow: 1.0,
        ..Default::default()
    }).unwrap();

    let cancel = taffy.new_leaf(Style {
        size: Size { width: Dimension::Length(102.96), height: Dimension::Length(42.0) },
        padding: Edges::all(LengthPercentage::Length(10.0)),
        margin: Edges::all(LengthPercentageAuto::Length(10.0)),
        ..Default::default()
    }).unwrap();

    let save = taffy.new_leaf(Style {
        size: Size { width: Dimension::Length(169.92), height: Dimension::Length(42.0) },
        padding: Edges::all(LengthPercentage::Length(10.0)),
        margin: Edges::all(LengthPercentageAuto::Length(10.0)),
        ..Default::default()
    }).unwrap();

    let row = taffy.new_with_children(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        size: Size { width: Dimension::Length(500.0), height: Dimension::Auto },
        ..Default::default()
    }, &[spacer, cancel, save]).unwrap();

    taffy.compute_layout(row, Size::MAX_CONTENT).unwrap();

    let r_cancel = taffy.layout(cancel).unwrap();
    println!("Cancel button location: {:?}, size: {:?}", r_cancel.location, r_cancel.size);

    let r_save = taffy.layout(save).unwrap();
    println!("Save button location: {:?}, size: {:?}", r_save.location, r_save.size);
    
    let r_row = taffy.layout(row).unwrap();
    println!("Row location: {:?}, size: {:?}", r_row.location, r_row.size);
}
