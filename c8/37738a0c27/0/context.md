# Session Context

## User Prompts

### Prompt 1

Fix CI issue, maybe a flaky test: https://github.com/moltis-org/moltis/actions/runs/24555667767

### Prompt 2

I'm using `just build-css` to build css, but that means `cargo build` does not work on its own. Since I rarelly change the CSS, would it make sense to commit the built assets for people to just run cargo build?

### Prompt 3

commiting crates/web/src/assets/style.css might generate tons of conflicts tho, because it's on a single line. Anything you could improve there?

