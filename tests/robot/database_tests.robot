*** Settings ***
Documentation     Database integration tests for nexo web server
Resource          resources.robot
Library           DatabaseLibrary
Library           RequestsLibrary
Suite Setup       Start Server
Suite Teardown    Stop Server

*** Variables ***
${DB_PATH}    data/db_test.sqlite

*** Test Cases ***
Database Connection
    [Documentation]    Test database connection and basic operations
    [Tags]    database    connection
    Connect To Database    sqlite3    ${DB_PATH}
    
    # Check if users table exists
    Table Must Exist    users
    
    # Check table structure
    @{columns}=    Query    PRAGMA table_info(users);
    Should Not Be Empty    ${columns}
    
    # Check if sessions table exists
    Table Must Exist    sessions
    
    Disconnect From Database

User Registration
    [Documentation]    Test user registration through API
    [Tags]    database    registration
    Wait For Server
    Create Session    nexo    ${SERVER_URL}
    
    # Test login with existing user
    ${headers}=    Create Dictionary    Content-Type=application/x-www-form-urlencoded
    ${data}=    Create Dictionary    username=thiago    password=1234
    ${response}=    POST On Session    nexo    /login    data=${data}    headers=${headers}
    Status Should Be    200    ${response}
    
    # Verify user exists in database
    Connect To Database    sqlite3    ${DB_PATH}
    @{result}=    Query    SELECT name FROM users WHERE name='thiago';
    Should Not Be Empty    ${result}
    Disconnect From Database

User Authentication
    [Documentation]    Test user authentication with database
    [Tags]    database    auth
    Wait For Server
    Create Session    nexo    ${SERVER_URL}
    
    # Test login with existing user
    ${headers}=    Create Dictionary    Content-Type=application/x-www-form-urlencoded
    ${data}=    Create Dictionary    username=thiago    password=testpass
    ${response}=    POST On Session    nexo    /login    data=${data}    headers=${headers}
    Status Should Be    200    ${response}
    
    # Verify response is successful (redirects to home page)
    Status Should Be    200    ${response}

*** Keywords ***
Setup Test Environment
    [Documentation]    Setup test environment
    Start Server
    Wait For Server

Teardown Test Environment
    [Documentation]    Cleanup test environment
    Stop Server

Clean Database
    [Documentation]    Clean test data from database
    Connect To Database    sqlite3    ${DB_PATH}
    Execute Sql String    DELETE FROM users WHERE username IN ('newuser', 'testuser');
    Disconnect From Database 