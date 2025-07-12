*** Settings ***
Documentation     API integration tests for nexo web server
Resource          resources.robot
Library           RequestsLibrary
Suite Setup       Start Server
Suite Teardown    Stop Server

*** Test Cases ***
Health Check Endpoint
    [Documentation]    Test the health check endpoint
    [Tags]    api    health
    Wait For Server
    Create Session    nexo    ${SERVER_URL}
    ${response}=    GET On Session    nexo    /health
    Status Should Be    200    ${response}
    ${json}=    Set Variable    ${response.json()}
    Should Contain    ${json}    status

Home Page Loads
    [Documentation]    Test that the home page loads correctly
    [Tags]    api    home
    Wait For Server
    Create Session    nexo    ${SERVER_URL}
    ${response}=    GET On Session    nexo    /
    Status Should Be    200    ${response}
    Should Contain    ${response.text}    <html

Login Endpoint
    [Documentation]    Test login functionality
    [Tags]    api    auth
    Wait For Server
    Create Session    nexo    ${SERVER_URL}
    ${headers}=    Create Dictionary    Content-Type=application/x-www-form-urlencoded
    ${data}=    Create Dictionary    username=test    password=test123
    ${response}=    POST On Session    nexo    /login    data=${data}    headers=${headers}
    Status Should Be    200    ${response}

Invalid Login
    [Documentation]    Test login with invalid credentials
    [Tags]    api    auth    negative
    Wait For Server
    Create Session    nexo    ${SERVER_URL}
    ${headers}=    Create Dictionary    Content-Type=application/x-www-form-urlencoded
    ${data}=    Create Dictionary    username=invalid    password=wrong
    ${response}=    POST On Session    nexo    /login    data=${data}    headers=${headers}
    Status Should Be    200    ${response}
    Should Contain    ${response.text}    Invalid username or password

*** Keywords ***
Setup Test Environment
    [Documentation]    Setup test environment
    Start Server
    Wait For Server

Teardown Test Environment
    [Documentation]    Cleanup test environment
    Stop Server 