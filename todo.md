# 0.2.0
- add request ids  https://github.com/imbolc/tower-request-id/blob/main/examples/logging.rs
- split functions into separate files and modules
- split request handler functions into separate sub functions
- add a password (basic auth) (the customer's zip) to make it a bit more secure
- Einspruchtemplate tests
- Einspruchtemplate multiple pages
- ausländische Nutzer? 

# 0.1.0
- how to get the tax office db on deploy?
- how to integrate the templates (separate repo) -
  - either git submodule or wget/curl before deploy? (in deploy script)
  - include at compile vs deliver with deployment
- Formulierungen + snippets für Widerspruch/Einspruch schreiben
    + Formulierung im Schreiben für Proforma Widerspruch/Einspruch: ???
      -> Begründung wird nachgereicht
- make sure all required data is available + fix missing
- test different cases (multiple page, 10 senders, ...)
- test at shuttle endpoint
- deploy to shuttle.rs
- test cases: 
  - 1. all possible fin plz+names -> should be 1 result each
  - 2. all possible PLZ (customer) to fin office -> Ausreißer
- Rust backend für PDF Erstellung
    + terra für template2html?
    + api with actix
    + queries für pdf,html,get
    + log meta info
    - log success
    + log errors
    + write payload into json
    + create html pages
      + create letter
      + create invoice
      + create index
      + create list/tips
    + create PDF files
    + prevent second empty page in PDFs
    + pages index, formresult
    - sql queries + tax office db integration
      - use the tax office data
      - how to handle multiple finance office results for same zip? like DUS 40476
    - create formresult page hashed email
      - separate page/endpoint for thank you page (redirect after form fill)
        - when creating the files also create a mapping hash(email) to file_id
        - the mapping should only be called once and then get deleted
        - the separate page should call the endpoint (poll until valid result) and get the actual URL for the overview and redirect to that