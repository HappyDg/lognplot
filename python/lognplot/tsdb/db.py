""" Time series database.
"""

from .series import ZoomSerie


class TsDb:
    """ A time series database.
    """

    def __init__(self):
        # TODO: load / store data in file!
        self._traces = {}  # The internal trace data.

    def get_or_create_serie(self, name):
        if name in self._traces:
            serie = self._traces[name]
        else:
            serie = ZoomSerie()
            self._traces[name] = serie
        return serie

    def add_sample(self, name, sample):
        serie = self.get_or_create_serie(name)
        serie.add_sample(sample)

    def add_samples(self, name, samples):
        serie = self.get_or_create_serie(name)
        serie.add_samples(samples)

    def query_len(self, name):
        serie = self.get_or_create_serie(name)
        return len(serie)

    def query_summary(self, name):
        serie = self.get_or_create_serie(name)
        return serie.metrics

    def query(self, name, timespan, count):
        """ Query the database on the given signal.
        """
        serie = self.get_or_create_serie(name)
        return serie.query(timespan, count)

    def query_metrics(self, name, timespan):
        serie = self.get_or_create_serie(name)
        return serie.query_metrics(timespan)
