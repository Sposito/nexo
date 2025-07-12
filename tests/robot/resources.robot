*** Settings ***
Documentation     Shared resources for nexo web server tests
Library           SeleniumLibrary
Library           RequestsLibrary
Library           DatabaseLibrary
Library           Process

*** Variables ***
${SERVER_URL}     http://localhost:8001
${BROWSER}        headlesschrome
${TIMEOUT}        10s

*** Keywords ***
Start Server
    [Documentation]    Start the nexo server for testing
    # Check if server is already running
    ${status}    ${value}=    Run Keyword And Ignore Error
    ...    GET    ${SERVER_URL}/health
    Run Keyword If    '${status}' == 'PASS'    Log    Server already running    level=INFO
    
    # If server is not running, fail the test
    Run Keyword If    '${status}' != 'PASS'    Fail    Server is not running. Please start the server manually with 'cargo run --release'
    
    # Set a dummy handle to avoid variable issues
    Set Suite Variable    ${SERVER_HANDLE}    None

Stop Server
    [Documentation]    Stop the nexo server
    # Since we're not managing the server process, just log that cleanup is skipped
    Log    Server cleanup skipped - server is managed externally    level=INFO

Wait For Server
    [Documentation]    Wait for server to be ready
    FOR    ${i}    IN RANGE    15
        ${status}    ${value}=    Run Keyword And Ignore Error
        ...    GET    ${SERVER_URL}/health
        Run Keyword If    '${status}' == 'PASS'    Exit For Loop
        Sleep    2s
    END
    Run Keyword If    '${status}' != 'PASS'    Fail    Server failed to start after 30 seconds. Check server.log for details.
    Should Be Equal    ${status}    PASS    Server not ready after 30 seconds

Clean Database
    [Documentation]    Clean test data from database
    Connect To Database    sqlite3    data/db_test.sqlite
    Execute Sql String    DELETE FROM users WHERE name IN ('newuser', 'testuser');
    Disconnect From Database 