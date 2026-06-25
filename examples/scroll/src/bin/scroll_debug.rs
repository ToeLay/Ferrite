use ferrite::prelude::*;

fn main() {
    let mut tree = LayoutTree::new();
    let mut items = Vec::new();
    for i in 1..=50 {
        items.push(
            row([
                text(&format!("Item #{}", i)).size(18.0),
                spacer(),
                button("Click Me", move || println!("Clicked item {}", i)),
            ])
            .padding(15.0)
            .align(AlignItems::Center)
        );
        if i < 50 {
            items.push(divider());
        }
    }

    let list = col(items).background(Color::rgb(0.15, 0.15, 0.15));
    
    let root_view = col([
        text("Scrollable List").size(24.0).padding(20.0),
        divider(),
        scroll(list),
    ])
    .background(Color::rgb(0.1, 0.1, 0.1))
    .flex_grow(1.0)
    .fill(); // Manually call fill()

    let root = root_view.build(&mut tree);
    let mut app = App::new(tree, root);
    app.render(400.0, 500.0);

    let root_layout = app.layout_tree().layout(app.root_node_id());
    
    let root_widget = app.root();
    let scroll_widget = root_widget.children()[2].as_ref();
    let scroll_layout = app.layout_tree().layout(scroll_widget.node_id());
    
    let child_widget = scroll_widget.children()[0].as_ref();
    let child_layout = app.layout_tree().layout(child_widget.node_id());

    println!("Root height: {}, Scroll height: {}, Child height: {}", root_layout.height, scroll_layout.height, child_layout.height);
}
