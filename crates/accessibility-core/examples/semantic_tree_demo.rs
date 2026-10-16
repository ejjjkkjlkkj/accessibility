use accessibility_core::tree::{
    next_focusable, previous_focusable, validate_tree, AccessibleNode, STATE_DISABLED,
    STATE_ENABLED, STATE_FOCUSABLE,
};
use accessibility_core::Role;

fn main() {
    let target = STATE_FOCUSABLE | STATE_ENABLED;
    let nodes = [
        AccessibleNode::new(1, None, Role::Window, 0, "Fenetre principale"),
        AccessibleNode::new(2, Some(1), Role::Heading, 0, "Lecteur d'ecran - test"),
        AccessibleNode::new(3, Some(1), Role::Button, target, "Demarrer"),
        AccessibleNode::new(
            4,
            Some(1),
            Role::Button,
            STATE_FOCUSABLE | STATE_DISABLED,
            "Desactive",
        ),
        AccessibleNode::new(5, Some(1), Role::Button, target, "Quitter"),
    ];

    if !validate_tree(&nodes) {
        eprintln!("FAIL: arbre semantique invalide");
        std::process::exit(1);
    }
    let next = next_focusable(&nodes, None);
    let after_first = next_focusable(&nodes, next);
    let previous = previous_focusable(&nodes, after_first);

    println!("ADM ACCESSIBILITY CORE DEMO");
    println!("Tree validation: PASS");
    println!("First focusable ID: {:?}", next);
    println!("Next focusable ID: {:?}", after_first);
    println!("Previous focusable ID: {:?}", previous);
    println!(
        "Disabled control skipped: {}",
        next == Some(3) && after_first == Some(5)
    );
    if next != Some(3) || after_first != Some(5) || previous != Some(3) {
        eprintln!("FAIL: navigation contract");
        std::process::exit(2);
    }
    println!("RESULT: PASS (semantic core only; no OS integration or speech output)");
}
