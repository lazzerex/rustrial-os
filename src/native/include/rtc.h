/**
 * Real-Time Clock (RTC) Driver Header
 */

#ifndef RTC_H
#define RTC_H

#include <stdint.h>

/**
 * Date and Time Structure
 */
typedef struct {
    uint16_t year;
    uint8_t month;
    uint8_t day;
    uint8_t hour;
    uint8_t minute;
    uint8_t second;
    uint8_t weekday;  // 1=Sunday, 2=Monday, etc.
} rtc_datetime_t;

// Function prototypes
void rtc_read_datetime(rtc_datetime_t* dt);
const char* rtc_weekday_str(uint8_t weekday);
const char* rtc_month_str(uint8_t month);

#endif // RTC_H
