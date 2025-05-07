use colored::Colorize;

pub fn show_diff(old_content: &str, new_content: &str) {
    // Use block-based diff to show only oh-my-dockers managed changes
    let old_managed = extract_managed_blocks(old_content);
    let new_managed = extract_managed_blocks(new_content);

    if old_managed == new_managed {
        println!("  {}", "No changes".yellow());
        return;
    }

    // Show removed blocks
    for block in &old_managed {
        if !new_managed.contains(block) {
            for line in block.lines() {
                println!("  {} {}", "-".red(), line.red());
            }
        }
    }

    // Show added blocks
    for block in &new_managed {
        if !old_managed.contains(block) {
            for line in block.lines() {
                println!("  {} {}", "+".green(), line.green());
            }
        }
    }
}

fn extract_managed_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current_block = String::new();

    for line in content.lines() {
        if line.trim().starts_with("# oh-my-dockers:") && !line.contains("end") {
            in_block = true;
            current_block.clear();
            current_block.push_str(line);
            current_block.push('\n');
        } else if line.trim() == "# oh-my-dockers: end" {
            current_block.push_str(line);
            blocks.push(current_block.clone());
            in_block = false;
        } else if in_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    blocks
}
