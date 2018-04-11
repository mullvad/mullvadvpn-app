#pragma once

#include <string>
#include <utility>
#include <vector>

class PropertyList
{
public:

	struct Property
	{
		Property(const std::wstring &n, const std::wstring &v)
			: name(n), value(v)
		{
		}

		Property(std::wstring &&n, std::wstring &&v)
			: name(n), value(v)
		{
		}

		std::wstring name;
		std::wstring value;
	};

	void add(const std::wstring &name, const std::wstring &value)
	{
		m_properties.emplace_back(name, value);
	}

	void add(std::wstring &&name, std::wstring &&value)
	{
		m_properties.emplace_back(name, value);
	}

	const std::vector<Property> &list()
	{
		return m_properties;
	}

	std::vector<Property>::const_iterator begin() const
	{
		return m_properties.begin();
	}

	std::vector<Property>::const_iterator end() const
	{
		return m_properties.end();
	}

private:

	std::vector<Property> m_properties;
};
