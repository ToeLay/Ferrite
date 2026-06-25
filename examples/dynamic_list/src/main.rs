use ferrite::prelude::*;

fn main() {
    let todos = use_state(vec!["Buy milk".to_string(), "Clean house".to_string(), "Walk dog".to_string()]);
    let new_todo = use_state("".to_string());

    ferrite::run(
        "Dynamic List Example",
        (600, 800),
        col([
            text("Todo List").size(32.0).color(Color::rgb(0.2, 0.2, 0.25)),
            
            row([
                input(new_todo, "What needs to be done?").width(300.0),
                button("Add", move || {
                    let text = new_todo.get();
                    if !text.is_empty() {
                        todos.push(text);
                        new_todo.set("".to_string());
                    }
                })
            ]).gap(12.0),
            
            divider(),

            list(todos, move |item: &String| {
                let item_clone = item.clone(); // Clone for the remove closure
                row([
                    text(&item_clone).size(20.0),
                    spacer(),
                    button("Remove", move || {
                        // Find the index of this item and remove it
                        let current_todos = todos.get();
                        if let Some(pos) = current_todos.iter().position(|x| x == &item_clone) {
                            todos.remove(pos);
                        }
                    })
                ]).align(AlignItems::Center).padding(8.0)
            }),
            
            spacer(),
            
            row([
                button("Clear All", move || todos.clear()),
                button("Reset", move || todos.set(vec!["Buy milk".to_string(), "Clean house".to_string(), "Walk dog".to_string()])),
            ]).gap(12.0)
        ])
        .padding(32.0)
        .gap(24.0)
    );
}
