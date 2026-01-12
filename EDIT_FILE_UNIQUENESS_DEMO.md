### Demo: Enhanced edit_file with uniqueness validation

Create a test file with multiple matches:
```
foo = "value1"
bar = foo + "suffix"  
foo = "value2"
```

#### Detailed match reporting:
When trying to replace "foo" without `replace_all`, the error now includes:
- Line and column positions of all matches
- Context lines around each match  
- Suggestions for making the target unique

#### Force selection modes:
```rust
use tachikoma_primitives::{EditFileOptions, MatchSelection};

// Force edit first match only
let opts = EditFileOptions::new().force_first();

// Force edit match at specific line
let opts = EditFileOptions::new().force_line(3);

// Force edit match by index
let opts = EditFileOptions::new().force_index(1);
```

#### Uniqueness checking API:
```rust
use tachikoma_primitives::check_uniqueness;

let result = check_uniqueness(content, "foo", 2); // 2 context lines
println!("Unique: {}", result.is_unique);
println!("Found {} matches", result.match_count);

for (i, m) in result.matches.iter().enumerate() {
    println!("Match {} at line {}, column {}", i+1, m.line, m.column);
    println!("{}", m.format_with_context());
}
```

All acceptance criteria have been implemented and tested!