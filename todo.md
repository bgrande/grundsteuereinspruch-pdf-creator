# 0.1.0
- Rust backend für PDF Erstellung
    - how to integrate the templates (separate repo) -
      - either git submodule or wget/curl before deploy? (in deploy script) 
      - include at compile vs deliver with deployment
    + terra für template2html?
    + api with actix
    + queries für pdf,html,get
    - log meta info
    - log success
    - log errors
    + write payload into json
    + create html pages
      + create letter
      + create invoice
      + create index
      + create list/tips
    - create PDF files
    - sql queries + tax office db integration
      - use the tax office data
    - make sure all required data is available + fix missing
    - deploy to shuttle
    - test at shuttle endpoint