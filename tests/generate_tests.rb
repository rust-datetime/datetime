#!/bin/env ruby

require 'date'
require 'json'

def fit(number)
  number.to_s.rjust(2, "0")
end

puts "["
foo = (2001..2022).map{|year|
  (01..52).map{|week|
    (01..07).map{|weekday|
      week = week.to_s.rjust(2,?0)
      weekday = weekday.to_s.rjust(1,?0)
      date_string = "#{year}-W#{week}-#{weekday}"
      date = Date.iso8601(date_string)
      puts "\
[\
\"#{date_string}\", \
[#{year},#{week.to_i},#{weekday}], \
\"#{date.year}-#{fit date.month}-#{fit date.day}\", \
[#{date.year},#{date.month},#{date.day}]],"

      [ [date_string], [[year], [week], [weekday]],
        "#{date.year}-#{fit date.month}-#{fit date.day}",
        [[date.year],[date.month],[date.day]] ]
    }
  }
}
puts "]"
#j =foo.to_json()
#puts JSON.pretty_generate(foo)
