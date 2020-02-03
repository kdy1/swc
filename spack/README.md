# TODOs
 - [ ] Namespace import
 - [ ] Chunking
 - [ ] Correct execution order
 - [ ] Dynamic imports
   - [x] Loading
 - [ ] Alias handling
 - [ ] Export filtering (in usage_analysis)

  
## TODOs (postponed)
 - [ ] Full require handling
   - [x] Loading
   - [ ] Optional dependency
   
 - [ ] Optional dependency

# Ideas




## Avoiding requires

We can merge all modules without side effects into a single file without hard work.
Identifier hygiene from `swc_ecma_transforms` should fit the use case.



## Maybe 

We can move all **pure** constants to top level function.
If those are marked with correct hygiene id, 
it will be resolved differently and can be removed by uglifyjs.


# Fixture tests

```yaml
  # Stores input files  
  - /input
    # Entries (this includes entry.js)
    - /input/entry*
  # Stores reference outputs
  - /output
    # Output entries
    - /output/entry*
    # Shared modules.
    - /output/chunk*
```