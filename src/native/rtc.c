/**
 * Real-Time Clock (RTC) Driver - C Implementation
 * 
 * Provides access to the CMOS/RTC chip for reading system date and time.
 */

#include "rtc.h"
#include <stdint.h>
#include <stdbool.h>

// CMOS/RTC I/O ports
#define CMOS_ADDRESS    0x70
#define CMOS_DATA       0x71

// CMOS Register addresses
#define RTC_SECONDS     0x00
#define RTC_MINUTES     0x02
#define RTC_HOURS       0x04
#define RTC_WEEKDAY     0x06
#define RTC_DAY         0x07
#define RTC_MONTH       0x08
#define RTC_YEAR        0x09
#define RTC_CENTURY     0x32
#define RTC_STATUS_A    0x0A
#define RTC_STATUS_B    0x0B

// I/O port access
static inline void outb(uint16_t port, uint8_t value) {
    __asm__ volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint8_t inb(uint16_t port) {
    uint8_t value;
    __asm__ volatile("inb %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

/**
 * Read a CMOS register
 */
static uint8_t rtc_read_register(uint8_t reg) {
    outb(CMOS_ADDRESS, reg | 0x80);  // Bit 7 disables NMI
    return inb(CMOS_DATA);
}

/**
 * Check if RTC update is in progress
 */
static bool rtc_is_updating(void) {
    return (rtc_read_register(RTC_STATUS_A) & 0x80) != 0;
}

/**
 * Wait for RTC update to complete
 */
static void rtc_wait_for_update(void) {
    while (rtc_is_updating()) {
        __asm__ volatile("pause");
    }
}

/**
 * Check if RTC is in 24-hour mode
 */
static bool rtc_is_24hour(void) {
    return (rtc_read_register(RTC_STATUS_B) & 0x02) != 0;
}

/**
 * Check if RTC values are in binary mode (vs BCD)
 */
static bool rtc_is_binary(void) {
    return (rtc_read_register(RTC_STATUS_B) & 0x04) != 0;
}

/**
 * Convert BCD to binary
 */
static uint8_t bcd_to_binary(uint8_t bcd) {
    return ((bcd >> 4) * 10) + (bcd & 0x0F);
}

/**
 * Read current date and time from RTC
 */
void rtc_read_datetime(rtc_datetime_t* dt) {
    if (!dt) return;
    
    // Wait for any update to complete
    rtc_wait_for_update();
    
    // Read all values
    uint8_t second = rtc_read_register(RTC_SECONDS);
    uint8_t minute = rtc_read_register(RTC_MINUTES);
    uint8_t hour = rtc_read_register(RTC_HOURS);
    uint8_t weekday = rtc_read_register(RTC_WEEKDAY);
    uint8_t day = rtc_read_register(RTC_DAY);
    uint8_t month = rtc_read_register(RTC_MONTH);
    uint8_t year = rtc_read_register(RTC_YEAR);
    uint8_t century = rtc_read_register(RTC_CENTURY);
    
    bool is_binary = rtc_is_binary();
    bool is_24h = rtc_is_24hour();
    
    // Convert from BCD if necessary
    if (!is_binary) {
        second = bcd_to_binary(second);
        minute = bcd_to_binary(minute);
        hour = bcd_to_binary(hour & 0x7F);  // Mask AM/PM bit
        day = bcd_to_binary(day);
        month = bcd_to_binary(month);
        year = bcd_to_binary(year);
        if (century != 0) {
            century = bcd_to_binary(century);
        }
    }
    
    // Handle 12-hour to 24-hour conversion
    if (!is_24h && (hour & 0x80)) {
        hour = ((hour & 0x7F) + 12) % 24;
    }
    
    // Calculate full year
    uint16_t full_year;
    if (century != 0) {
        full_year = (uint16_t)century * 100 + (uint16_t)year;
    } else {
        // Assume 21st century
        full_year = 2000 + (uint16_t)year;
    }
    
    // Fill in the structure
    dt->year = full_year;
    dt->month = month;
    dt->day = day;
    dt->hour = hour;
    dt->minute = minute;
    dt->second = second;
    dt->weekday = weekday;
}

/**
 * Get weekday as string
 */
const char* rtc_weekday_str(uint8_t weekday) {
    static const char* days[] = {
        "Unknown", "Sunday", "Monday", "Tuesday", 
        "Wednesday", "Thursday", "Friday", "Saturday"
    };
    return (weekday <= 7) ? days[weekday] : days[0];
}

/**
 * Get month as string
 */
const char* rtc_month_str(uint8_t month) {
    static const char* months[] = {
        "Unknown", "January", "February", "March", "April",
        "May", "June", "July", "August", "September",
        "October", "November", "December"
    };
    return (month >= 1 && month <= 12) ? months[month] : months[0];
}
