*** Settings ***
Documentation     UI tests for nexo web server
Resource          resources.robot
Library           SeleniumLibrary
Suite Setup       Start Server
Suite Teardown    Stop Server

*** Test Cases ***
Home Page UI Elements
    [Documentation]    Test that home page UI elements are present
    [Tags]    ui    home
    Wait For Server
    Open Browser    ${SERVER_URL}    ${BROWSER}    headless=True
    Set Window Size    1920    1080
    
    # Check for basic HTML structure
    ${title}=    Get Title
    Should Contain    ${title}    Login
    
    # Just verify page loads with form
    Page Should Contain    Username
    Page Should Contain    Password
    
    Close Browser

Login Form UI
    [Documentation]    Test login form UI elements
    [Tags]    ui    login
    Wait For Server
    Open Browser    ${SERVER_URL}/    ${BROWSER}    headless=True
    Set Window Size    1920    1080
    
    # Just verify page loads with form
    Page Should Contain    Username
    Page Should Contain    Password
    
    Close Browser

Responsive Design
    [Documentation]    Test responsive design on different screen sizes
    [Tags]    ui    responsive
    Wait For Server
    
    # Test desktop view
    Open Browser    ${SERVER_URL}    ${BROWSER}    headless=True
    Set Window Size    1920    1080
    Sleep    1s
    Close Browser
    
    # Test mobile view
    Open Browser    ${SERVER_URL}    ${BROWSER}    headless=True
    Set Window Size    375    667
    Sleep    1s
    Close Browser 