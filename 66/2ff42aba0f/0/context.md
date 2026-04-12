# Session Context

## User Prompts

### Prompt 1

can you use the feedback from https://github.com/moltis-org/moltis/discussions/680 to use in website/ as another feedback? It's a good one I think.

### Prompt 2

can you make feedbacks horizontally scrollable so I can add more without taking much height

### Prompt 3

commit and push

### Prompt 4

find the largest rust code files in moltis, give me the 10 biggest files.

### Prompt 5

What's the usual max size per rust file we should enforce, and then split those files per domain? I'd like to enforce a strict rule, including a CI job or clippy/fmt rule.

### Prompt 6

yes proceed, 1,500 is good for now but out of curiosity, how many files are over 1k lines?

### Prompt 7

I added prompts in .gitignore so why do I see files in prompts committed to git? Can you check if any has private keys and tokens? I feel like it does , like the NOSTR PR

### Prompt 8

remove them from git now it's fine, push it

### Prompt 9

I still see files:

~/t/m/moltis main [!] ❯ ls -l prompts
total 1144
-rw-r--r--    1 penso  staff   1665 Jan 31 06:01 2026-01-30-plan-move-providers-to-a-dedicated-nav-page.md
-rw-r--r--    1 penso  staff   3167 Jan 31 01:44 2026-01-30-plan-multi-page-ui-with-cron-management.md
-rw-r--r--    1 penso  staff   9494 Jan 31 01:43 2026-01-30-plan-projects-feature-for-moltis.md
-rw-r--r--@   1 penso  staff    901 Feb  1 08:24 2026-02-01-fix-ci-failures.md
-rw-r--r--@   1 penso  staff   8745 Feb  5 ...

