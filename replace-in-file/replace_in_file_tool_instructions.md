## replace_in_file

The `replace_in_file` tool is your primary method for editing files. It uses SEARCH/REPLACE blocks to make targeted changes:

```
------- SEARCH
[exact content to find]
=======
[new content to replace with]
+++++++ REPLACE
```

**Critical rules for replace_in_file:**
1. SEARCH content must match the file section EXACTLY (character-for-character, including whitespace)
2. SEARCH/REPLACE blocks only replace the first match occurrence
3. Use multiple SEARCH/REPLACE blocks for multiple changes, listed in file order
4. Keep blocks concise - include just the changing lines plus a few surrounding lines for uniqueness
5. Each line must be complete (never truncate mid-line)
6. To delete code, use an empty REPLACE section
7. To move code, use two blocks (delete from original + insert at new location)