*** Settings ***
Documentation     Complete test suite for nexo web server
Resource          resources.robot
Suite Setup       Start Server
Suite Teardown    Stop Server

*** Test Cases ***
Complete Integration Test
    [Documentation]    Run complete integration test suite
    [Tags]    integration    smoke
    
    # This test just verifies the server is running
    Wait For Server
    Create Session    nexo    ${SERVER_URL}
    ${response}=    GET On Session    nexo    /health
    Status Should Be    200    ${response}

Performance Test
    [Documentation]    Test basic performance metrics
    [Tags]    performance
    Wait For Server
    
    Create Session    nexo    ${SERVER_URL}
    
    # Test response time
    ${start_time}=    Get Time    epoch
    ${response}=    GET On Session    nexo    /health
    ${end_time}=    Get Time    epoch
    ${response_time}=    Evaluate    ${end_time} - ${start_time}
    
    # Response should be under 1 second
    Should Be True    ${response_time} < 1
    
    Status Should Be    200    ${response}

Security Test
    [Documentation]    Test basic security measures
    [Tags]    security
    Wait For Server
    
    Create Session    nexo    ${SERVER_URL}
    
    # Test SQL injection attempt
    ${headers}=    Create Dictionary    Content-Type=application/x-www-form-urlencoded
    ${data}=    Create Dictionary    username=admin'--    password=test
    ${response}=    POST On Session    nexo    /login    data=${data}    headers=${headers}
    Status Should Be    200    ${response}
    Should Contain    ${response.text}    Invalid username or password
    
    # Test XSS attempt
    ${data}=    Create Dictionary    username=<script>alert('xss')</script>    password=test
    ${response}=    POST On Session    nexo    /login    data=${data}    headers=${headers}
    Status Should Be    200    ${response}
    Should Contain    ${response.text}    Invalid username or password 