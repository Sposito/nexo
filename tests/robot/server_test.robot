*** Settings ***
Documentation     Server startup and health check tests
Resource          resources.robot
Library           RequestsLibrary
Suite Setup       Start Server
Suite Teardown    Stop Server

*** Test Cases ***
Server Starts Successfully
    [Documentation]    Verify server starts and responds to health check
    [Tags]    smoke    server
    Wait For Server
    
    # Test health endpoint
    Create Session    nexo    ${SERVER_URL}
    ${response}=    GET On Session    nexo    /health
    Status Should Be    200    ${response}
    
    # Verify response structure
    ${json}=    Set Variable    ${response.json()}
    Should Contain    ${json}    status
    Should Be Equal    ${json}[status]    ok

Server Responds To Root Endpoint
    [Documentation]    Verify server responds to root endpoint
    [Tags]    smoke    server
    Wait For Server
    
    Create Session    nexo    ${SERVER_URL}
    ${response}=    GET On Session    nexo    /
    Status Should Be    200    ${response}
    Should Contain    ${response.text}    <html 