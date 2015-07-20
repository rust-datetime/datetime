#!/bin/env ruby

require 'date'
require 'json'

def fit(number)
  number.to_s.rjust(2, ?0)
end

foo = []
(2001..2022).map{|year|
  (01..52).map{|week|
    (01..07).map{|weekday|
      date_string = "#{year}-W#{fit week}-#{weekday}"
      date = Date.iso8601(date_string)

      foo.push   [  date_string , [ year ,  week ,  weekday ],
        "#{date.year}-#{date.month}-#{fit date.day}",
        [date.year, date.month , date.day ] ]
    }
  }
}
puts foo.to_json()
